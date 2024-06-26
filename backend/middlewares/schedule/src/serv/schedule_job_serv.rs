use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use std::vec;

use bios_sdk_invoke::clients::base_spi_client::BaseSpiClient;
use bios_sdk_invoke::clients::spi_kv_client::{KvItemDetailResp, SpiKvClient};
use bios_sdk_invoke::clients::spi_log_client::{LogItemFindReq, SpiLogClient};
use bios_sdk_invoke::invoke_enumeration::InvokeModuleKind;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::chrono::{self, TimeDelta, Utc};
use tardis::log::{error, info, trace, warn};
use tardis::tokio::sync::RwLock;
use tardis::tokio::time;
use tardis::web::web_resp::{TardisPage, TardisResp};
use tardis::{serde_json, TardisFuns, TardisFunsInst};
use tsuki_scheduler::prelude::*;

use crate::dto::schedule_job_dto::{KvScheduleJobItemDetailResp, ScheduleJobAddOrModifyReq, ScheduleJobInfoResp, ScheduleJobKvSummaryResp, ScheduleTaskInfoResp};
use crate::schedule_config::ScheduleConfig;
use crate::schedule_constants::{DOMAIN_CODE, KV_KEY_CODE};

/// global service instance
/// 全局服务实例
static GLOBAL_SERV: OnceLock<Arc<OwnedScheduleTaskServ>> = OnceLock::new();

/// get service instance without checking if it's initialized
/// # Safety
/// if called before init, this function will panic
///
/// 获取服务实例，不检查是否初始化
/// # 安全性
/// 如果在初始化之前调用，此函数将会panic
fn service() -> Arc<OwnedScheduleTaskServ> {
    GLOBAL_SERV.get().expect("trying to get scheduler before it's initialized").clone()
}

/// still not good, should manage to merge it with `OwnedScheduleTaskServ::add`
/// same as `delete`
pub(crate) async fn add_or_modify(add_or_modify: ScheduleJobAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let code = add_or_modify.code.to_string();
    // if exist delete it first
    // 如果存在，先删除
    if service().code_uuid.write().await.get(&code).is_some() {
        delete(&code, funs, ctx).await?;
    }
    // 1. log add operation
    // 1. 记录添加操作
    SpiLogClient::add(
        "schedule_job",
        "add job",
        None,
        None,
        Some(code.to_string()),
        Some("add".to_string()),
        None,
        Some(Utc::now().to_rfc3339()),
        None,
        None,
        funs,
        ctx,
    )
    .await?;
    let config = funs.conf::<ScheduleConfig>();
    // 2. sync to kv
    // 2. 同步到kv
    SpiKvClient::add_or_modify_item(&format!("{KV_KEY_CODE}{code}"), &add_or_modify, None, None, funs, ctx).await?;
    // 3. notify cache
    // 3. 通知缓存
    let mut conn = funs.cache().cmd().await?;
    let cache_key_job_changed_info = &config.cache_key_job_changed_info;
    conn.set_ex(&format!("{cache_key_job_changed_info}{code}"), "update", config.cache_key_job_changed_timer_sec as u64).await?;
    // 4. do add at local scheduler
    // 4. 在本地调度器中添加
    ScheduleTaskServ::add(add_or_modify, &config).await?;
    Ok(())
}

pub(crate) async fn delete(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // 1. log this operation
    SpiLogClient::add(
        "schedule_job",
        "delete job",
        None,
        None,
        Some(code.to_string()),
        Some("delete".to_string()),
        None,
        Some(Utc::now().to_rfc3339()),
        None,
        None,
        funs,
        ctx,
    )
    .await?;
    // 2. sync to kv
    SpiKvClient::delete_item(&format!("{KV_KEY_CODE}{code}"), funs, ctx).await?;
    // 3. notify cache
    let config = funs.conf::<ScheduleConfig>();
    let mut conn = funs.cache().cmd().await?;
    let cache_key_job_changed_info = &config.cache_key_job_changed_info;
    conn.set_ex(&format!("{cache_key_job_changed_info}{code}"), "delete", config.cache_key_job_changed_timer_sec as u64).await?;
    // 4. do delete at local scheduler
    if service().code_uuid.read().await.get(code).is_some() {
        // delete schedual-task from scheduler
        // 从调度器中删除调度任务
        ScheduleTaskServ::delete(code).await?;
    }
    Ok(())
}

pub(crate) async fn find_job(code: Option<String>, page_number: u32, page_size: u16, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<ScheduleJobInfoResp>> {
    let kv_url = BaseSpiClient::module_url(InvokeModuleKind::Kv, funs).await?;
    let headers = BaseSpiClient::headers(None, funs, ctx).await?;
    let resp = funs
        .web_client()
        .get::<TardisResp<TardisPage<ScheduleJobKvSummaryResp>>>(
            &format!(
                "{}/ci/item/match?key_prefix={}&page_number={}&page_size={}",
                kv_url,
                format_args!("{}{}", KV_KEY_CODE, code.unwrap_or("".to_string())),
                page_number,
                page_size
            ),
            headers,
        )
        .await?;
    let body = BaseSpiClient::package_resp(resp)?;
    let Some(pages) = body else {
        return Err(funs.err().conflict("find_job", "find", "get Job Kv failed", ""));
    };
    Ok(TardisPage {
        page_size: pages.page_size,
        page_number: pages.page_number,
        total_size: pages.total_size,
        records: pages
            .records
            .into_iter()
            .map(|record| {
                let job = ScheduleJobAddOrModifyReq::parse_from_json(&record.value);
                ScheduleJobInfoResp {
                    code: record.key.replace(KV_KEY_CODE, ""),
                    cron: job.cron,
                    callback_url: job.callback_url,
                    callback_headers: job.callback_headers,
                    callback_method: job.callback_method,
                    callback_body: job.callback_body,
                    create_time: Some(record.create_time),
                    update_time: Some(record.update_time),
                    enable_time: job.enable_time,
                    disable_time: job.disable_time,
                }
            })
            .collect(),
    })
}

pub(crate) async fn find_one_job(code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<KvScheduleJobItemDetailResp>> {
    let kv_url = BaseSpiClient::module_url(InvokeModuleKind::Kv, funs).await?;
    let headers = BaseSpiClient::headers(None, funs, ctx).await?;
    let resp = funs.web_client().get::<TardisResp<Option<KvItemDetailResp>>>(&format!("{}/ci/item?key={}", kv_url, format_args!("{}{}", KV_KEY_CODE, code)), headers).await?;
    let body = BaseSpiClient::package_resp(resp)?;
    body.flatten().map(KvScheduleJobItemDetailResp::try_from).transpose()
}

pub(crate) async fn find_task(
    job_code: &str,
    ts_start: Option<chrono::DateTime<Utc>>,
    ts_end: Option<chrono::DateTime<Utc>>,
    page_number: u32,
    page_size: u16,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<ScheduleTaskInfoResp>> {
    let resp = SpiLogClient::find(
        LogItemFindReq {
            tag: "schedule_task".to_string(),
            keys: Some(vec![job_code.into()]),
            page_number,
            page_size: page_size * 2,
            ts_start,
            ts_end,
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?;
    let Some(page) = resp else {
        return Err(funs.err().conflict("find_job", "find", "task response missing body", ""));
    };
    let mut records = vec![];
    let mut log_iter = page.records.into_iter();
    while let Some(start_log) = log_iter.next() {
        let mut task = ScheduleTaskInfoResp {
            start: Some(start_log.ts),
            end: None,
            err_msg: None,
        };
        if let Some(end_log) = log_iter.next() {
            task.end = Some(end_log.ts);
            task.err_msg = Some(end_log.content);
        }
        records.push(task)
    }

    Ok(TardisPage {
        page_number: page.page_number,
        page_size: page.page_size / 2,
        total_size: page.total_size / 2,
        records,
    })
}

pub(crate) async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let service_instance = OwnedScheduleTaskServ::init(funs, ctx).await?;
    GLOBAL_SERV.get_or_init(|| service_instance);
    Ok(())
}

pub struct ScheduleTaskServ;

impl ScheduleTaskServ {
    /// add schedule task
    pub async fn add(add_or_modify: ScheduleJobAddOrModifyReq, config: &ScheduleConfig) -> TardisResult<()> {
        service().add(add_or_modify, config).await
    }

    pub async fn delete(code: &str) -> TardisResult<()> {
        service().delete(code).await
    }
}

#[derive(Clone)]
pub struct OwnedScheduleTaskServ {
    #[allow(clippy::type_complexity)]
    pub code_uuid: Arc<RwLock<HashMap<String, TaskUid>>>,
    pub client: AsyncSchedulerClient<Tokio>,
}

impl OwnedScheduleTaskServ {
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Arc<Self>> {
        let cache_client = funs.cache();
        let scheduler = AsyncSchedulerRunner::tokio();
        let client = scheduler.client();
        tardis::tokio::spawn(async move {
            let _ = scheduler.run().await;
        });
        let code_uuid_cache_raw = Arc::new(RwLock::new(HashMap::<String, TaskUid>::new()));
        let serv_raw = Arc::new(Self {
            code_uuid: code_uuid_cache_raw,
            client,
        });
        let serv = serv_raw.clone();
        let sync_db_ctx = ctx.clone();
        // sync from db task
        tardis::tokio::spawn(async move {
            let funs = TardisFuns::inst(DOMAIN_CODE, None);
            // every 5 seconds, query if webserver is started
            let mut interval = time::interval(Duration::from_secs(5));
            let config = funs.conf::<ScheduleConfig>().clone();
            let mut retry_time = 0;
            let max_retry_time = 5;
            loop {
                if TardisFuns::web_server().is_running().await {
                    if let Ok(job_resp) = self::find_job(None, 1, 9999, &funs, &sync_db_ctx).await {
                        let jobs = job_resp.records;
                        for job in jobs {
                            serv.add(job.create_add_or_mod_req(), &config).await.map_err(|e| error!("fail to add schedule task: {e}")).unwrap_or_default();
                        }
                        info!("synced all jobs from kv");
                        break;
                    } else {
                        warn!("encounter an error while init schedule middlewares: fail to find job {retry_time}/{max_retry_time}");
                        retry_time += 1;
                        if retry_time >= max_retry_time {
                            error!("fail to sync jobs from kv, schedule running without history jobs");
                            break;
                        }
                    }
                }
                interval.tick().await;
            }
        });
        let ctx = ctx.clone();
        let serv = serv_raw.clone();
        // sync from cache task
        tardis::tokio::spawn(async move {
            let funs = TardisFuns::inst(DOMAIN_CODE, None);
            let config = funs.conf::<ScheduleConfig>();
            let mut interval = time::interval(Duration::from_secs(config.cache_key_job_changed_timer_sec as u64));
            loop {
                let mut conn = cache_client.cmd().await;
                let mut res_iter = {
                    match conn {
                        Ok(ref mut cache_cmd) => match cache_cmd.scan_match::<_, String>(&format!("{}*", config.cache_key_job_changed_info)).await {
                            Ok(res_iter) => res_iter,
                            Err(e) => {
                                error!("fail to scan match in redis: {e}");
                                break;
                            }
                        },
                        Err(e) => {
                            error!("fail to get redis connection: {e}");
                            break;
                        }
                    }
                };
                trace!("[Schedule] Fetch changed Job cache");
                {
                    // collect configs from remote cache
                    while let Some(remote_job_code) = res_iter.next_item().await {
                        {
                            let code = remote_job_code.trim_start_matches(&config.cache_key_job_changed_info);
                            let funs = TardisFuns::inst(DOMAIN_CODE, None);
                            match self::find_one_job(code, &funs, &ctx).await {
                                Ok(Some(resp)) => {
                                    // if we have this job code in local cache, update or add it
                                    serv.add(resp.value, &config).await.map_err(|e| error!("fail to add schedule task: {e}")).unwrap_or_default();
                                }
                                Ok(None) => {
                                    // if we don't have this job code in local cache, remove it
                                    serv.delete(&remote_job_code).await.map_err(|e| error!("fail to delete schedule task: {e}")).unwrap_or_default();
                                }
                                Err(e) => {
                                    error!("fail to fetch error from spi-kv: {e}")
                                }
                            }
                        }
                    }
                }
                interval.tick().await;
            }
        });
        Ok(serv_raw)
    }

    /// generate distributed lock key for a certain task
    fn gen_distributed_lock_key(code: &str, config: &ScheduleConfig) -> String {
        format!("{}{}", config.distributed_lock_key_prefix, code)
    }

    /// add schedule task
    pub async fn add(&self, job_config: ScheduleJobAddOrModifyReq, config: &ScheduleConfig) -> TardisResult<()> {
        let has_job = { self.code_uuid.read().await.get::<String>(job_config.code.as_ref()).is_some() };
        if has_job {
            self.delete(&job_config.code).await?;
        }
        let callback_req = job_config.build_request()?;
        let code = job_config.code.to_string();
        let ctx = TardisContext {
            own_paths: "".to_string(),
            ak: "".to_string(),
            owner: "".to_string(),
            roles: vec![],
            groups: vec![],
            ..Default::default()
        };
        let lock_key = OwnedScheduleTaskServ::gen_distributed_lock_key(&code, config);
        let distributed_lock_expire_sec = config.distributed_lock_expire_sec;
        let enable_time = job_config.enable_time;
        let disable_time = job_config.disable_time;
        // startup cron scheduler
        let code = code.clone();
        let ctx = ctx.clone();
        let lock_key = lock_key.clone();
        let mut schedule_builder = job_config.cron.iter().filter_map(|cron| Cron::local_from_cron_expr(cron).ok()).fold(ScheduleDynBuilder::default(), ScheduleDynBuilder::or);
        if let Some(enable_time) = enable_time {
            schedule_builder = schedule_builder.after(enable_time);
        }
        if let Some(disable_time) = disable_time {
            if disable_time < Utc::now() {
                return Ok(());
            }
            schedule_builder = schedule_builder.before(disable_time);
        }
        schedule_builder = schedule_builder.throttling(TimeDelta::minutes(1));
        let task = Task::tokio(schedule_builder, move || {
            let callback_req = callback_req.try_clone().expect("body should be a string");
            let code = code.clone();
            let lock_key = lock_key.clone();
            let ctx = ctx.clone();

            async move {
                let cache_client = TardisFuns::cache();
                let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
                // about set and setnx, see:
                // 1. https://redis.io/commands/set/
                // 2. https://redis.io/commands/setnx/
                // At Redis version 2.6.12, setnx command is regarded as deprecated. see: https://redis.io/commands/setnx/
                // "executing" could be any string now, it's just a placeholder
                match cache_client.set_nx(&lock_key, "executing").await {
                    Ok(true) => {
                        // safety: it's ok to unwrap in this closure, scheduler will restart this job when after panic
                        let Ok(()) = cache_client.expire(&lock_key, distributed_lock_expire_sec as i64).await else {
                            return;
                        };
                        trace!("executing schedule task {code}");
                        // 1. write log exec start
                        let Ok(_) = SpiLogClient::add(
                            "schedule_task",
                            format!("schedule task {} exec start", code).as_str(),
                            None,
                            None,
                            Some(code.to_string()),
                            Some("exec-start".to_string()),
                            Some(tardis::chrono::Utc::now().to_rfc3339()),
                            Some(Utc::now().to_rfc3339()),
                            None,
                            None,
                            &funs,
                            &ctx,
                        )
                        .await
                        else {
                            return;
                        };
                        // 2. request webhook
                        match TardisFuns::web_client().raw().execute(callback_req).await {
                            Ok(resp) => {
                                let status_code = resp.status();
                                let remote_addr = resp.remote_addr().as_ref().map(SocketAddr::to_string);
                                let response_header: HashMap<String, String> = resp
                                    .headers()
                                    .into_iter()
                                    .filter_map(|(k, v)| {
                                        let v = v.to_str().ok()?.to_string();
                                        Some((k.to_string(), v))
                                    })
                                    .collect();
                                let ext = serde_json::json! {
                                    {
                                        "remote_addr": remote_addr,
                                        "status_code": status_code.to_string(),
                                        "headers": response_header
                                    }
                                };
                                let content = resp.text().await.unwrap_or_default();
                                // 3.1. write log exec end
                                let Ok(_) = SpiLogClient::add(
                                    "schedule_task",
                                    &content,
                                    Some(ext),
                                    None,
                                    Some(code.to_string()),
                                    Some("exec-end".to_string()),
                                    None,
                                    Some(Utc::now().to_rfc3339()),
                                    None,
                                    None,
                                    &funs,
                                    &ctx,
                                )
                                .await
                                else {
                                    return;
                                };
                            }
                            Err(e) => {
                                // 3.2. write log exec end
                                let Ok(_) = SpiLogClient::add(
                                    "schedule_task",
                                    &e.to_string(),
                                    None,
                                    None,
                                    Some(code.to_string()),
                                    Some("exec-fail".to_string()),
                                    None,
                                    Some(Utc::now().to_rfc3339()),
                                    None,
                                    None,
                                    &funs,
                                    &ctx,
                                )
                                .await
                                else {
                                    return;
                                };
                            }
                        }
                        trace!("executed schedule task {code}");
                    }
                    Ok(false) => {
                        trace!("schedule task {} is executed by other nodes, skip", code);
                    }
                    Err(e) => {
                        error!("cannot set lock to schedule task {code}, error: {e}");
                    }
                }
            }
        });
        let task_uid = TaskUid::uuid();
        self.client.add_task(task_uid, task);
        {
            self.code_uuid.write().await.insert(job_config.code.to_string(), task_uid);
        }
        Ok(())
    }

    pub async fn delete(&self, code: &str) -> TardisResult<()> {
        let mut uuid_cache = self.code_uuid.write().await;
        if let Some(uid) = uuid_cache.get(code) {
            self.client.remove_task(*uid);
            uuid_cache.remove(code);
        }
        Ok(())
    }
}
