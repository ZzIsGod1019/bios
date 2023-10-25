use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_filer_dto::RbumBasicFilterReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    rbum_initializer,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_item_serv::RbumItemCrudOperation, rbum_kind_serv::RbumKindServ},
};
use bios_sdk_invoke::invoke_initializer;

use itertools::Itertools;
use serde_json::Value;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::{
        reldb_client::TardisActiveModel,
        sea_orm::{
            self,
            sea_query::{Query, Table},
        },
    },
    log::info,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{
        cc::{flow_cc_inst_api, flow_cc_model_api, flow_cc_state_api},
        ci::flow_ci_inst_api,
        cs::flow_cs_config_api,
    },
    domain::{flow_config, flow_inst, flow_model, flow_state, flow_transition},
    dto::{
        flow_model_dto::FlowModelFilterReq,
        flow_state_dto::FlowSysStateKind,
        flow_transition_dto::{
            FlowTransitionActionByVarChangeInfoChangedKind, FlowTransitionActionChangeInfo, FlowTransitionActionChangeKind, FlowTransitionDoubleCheckInfo, FlowTransitionInitInfo,
        },
    },
    flow_config::{BasicInfo, FlowBasicInfoManager, FlowConfig},
    flow_constants::{self, DOMAIN_CODE},
    serv::{flow_inst_serv::FlowInstServ, flow_model_serv::FlowModelServ},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    init_db(funs).await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            flow_constants::DOMAIN_CODE,
            (
                flow_cc_state_api::FlowCcStateApi,
                flow_cc_model_api::FlowCcModelApi,
                flow_cc_inst_api::FlowCcInstApi,
                flow_cs_config_api::FlowCsConfigApi,
                flow_ci_inst_api::FlowCiInstApi,
            ),
        )
        .await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst) -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<FlowConfig>().rbum.clone()).await?;
    invoke_initializer::init(funs.module_code(), funs.conf::<FlowConfig>().invoke.clone())?;
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    funs.begin().await?;
    if check_initialized(&funs, &ctx).await? {
        init_basic_info(&funs).await?;
        self::modify_post_actions(&funs, &ctx).await?;
        self::check_data(&funs, &ctx).await?;
    } else {
        let db_kind = TardisFuns::reldb().backend();
        let compatible_type = TardisFuns::reldb().compatible_type();
        funs.db().init(flow_state::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_model::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_transition::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_inst::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_config::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        init_rbum_data(&funs, &ctx).await?;
    };
    funs.commit().await?;
    Ok(())
}

async fn check_initialized(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    RbumDomainServ::exist_rbum(
        &RbumBasicFilterReq {
            ignore_scope: true,
            rel_ctx_owner: false,
            code: Some(flow_constants::DOMAIN_CODE.to_string()),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await
}

async fn init_basic_info<'a>(funs: &TardisFunsInst) -> TardisResult<()> {
    let kind_state_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_STATE_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found state kind", ""))?;
    let kind_model_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_MODEL_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found model kind", ""))?;

    let domain_flow_id =
        RbumDomainServ::get_rbum_domain_id_by_code(flow_constants::DOMAIN_CODE, funs).await?.ok_or_else(|| funs.err().not_found("flow", "init", "not found flow domain", ""))?;

    FlowBasicInfoManager::set(BasicInfo {
        kind_state_id,
        kind_model_id,
        domain_flow_id,
    })?;
    Ok(())
}

// @TODO temporary
pub async fn modify_post_actions(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    #[derive(sea_orm::FromQueryResult)]
    pub struct FlowTransactionPostAction {
        id: String,
        action_by_post_changes: Value,
    }
    let transactions = funs
        .db()
        .find_dtos::<FlowTransactionPostAction>(
            Query::select()
                .columns([
                    (flow_transition::Entity, flow_transition::Column::Id),
                    (flow_transition::Entity, flow_transition::Column::ActionByPostChanges),
                ])
                .from(flow_transition::Entity),
        )
        .await?
        .into_iter()
        .filter(|res| !TardisFuns::json.json_to_obj::<Vec<FlowTransitionActionChangeInfo>>(res.action_by_post_changes.clone()).unwrap_or_default().is_empty())
        .collect_vec();
    for transaction in transactions {
        let mut post_changes = TardisFuns::json.json_to_obj::<Vec<FlowTransitionActionChangeInfo>>(transaction.action_by_post_changes.clone()).unwrap_or_default();
        for post_change in post_changes.iter_mut() {
            if post_change.changed_kind.is_none() && post_change.kind == FlowTransitionActionChangeKind::Var {
                if post_change.changed_val.is_some() {
                    post_change.changed_kind = Some(FlowTransitionActionByVarChangeInfoChangedKind::ChangeContent);
                } else {
                    post_change.changed_kind = Some(FlowTransitionActionByVarChangeInfoChangedKind::Clean);
                }
            }
        }
        let flow_transition = flow_transition::ActiveModel {
            id: sea_orm::ActiveValue::Set(transaction.id.clone()),
            action_by_post_changes: sea_orm::ActiveValue::Set(TardisFuns::json.obj_to_json(&post_changes)?),
            ..Default::default()
        };
        funs.db().update_one(flow_transition, ctx).await?;
    }

    Ok(())
}

pub async fn init_rbum_data(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let kind_state_id = add_kind(flow_constants::RBUM_KIND_STATE_CODE, flow_constants::RBUM_EXT_TABLE_STATE, funs, ctx).await?;
    let kind_model_id = add_kind(flow_constants::RBUM_KIND_MODEL_CODE, flow_constants::RBUM_EXT_TABLE_MODEL, funs, ctx).await?;

    let domain_flow_id = add_domain(funs, ctx).await?;

    FlowBasicInfoManager::set(BasicInfo {
        kind_state_id,
        kind_model_id,
        domain_flow_id,
    })?;

    info!("Flow initialization is complete.",);
    Ok(())
}

async fn add_kind<'a>(scheme: &str, ext_table: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(scheme.to_string()),
            name: TrimString(scheme.to_string()),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some(ext_table.to_string().to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await
}

async fn add_domain<'a>(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(flow_constants::DOMAIN_CODE.to_string()),
            name: TrimString(flow_constants::DOMAIN_CODE.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await
}

pub async fn truncate_data<'a>(funs: &TardisFunsInst) -> TardisResult<()> {
    rbum_initializer::truncate_data(funs).await?;
    funs.db().execute(Table::truncate().table(flow_state::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_model::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_transition::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_inst::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_config::Entity)).await?;
    funs.cache().flushdb().await?;
    Ok(())
}

// @TODO temporary
async fn check_data(funs: &TardisFunsInst, global_ctx: &TardisContext) -> TardisResult<()> {
    //1.add missed model
    let proj_insts = FlowInstServ::paginate(None, Some("PROJ".to_string()), Some(false), Some(true), 1, 99999, funs, global_ctx).await?;
    for proj_inst in proj_insts.records {
        let ctx = TardisContext {
            own_paths: proj_inst.own_paths.clone(),
            owner: proj_inst.create_ctx.owner.clone(),
            ..global_ctx.clone()
        };
        FlowModelServ::get_models(vec!["MS", "REQ", "ITER", "TASK", "CTS", "TP", "ISSUE", "TS"], None, funs, &ctx).await?;
    }
    TardisFuns::reldb_by_module_or_default(DOMAIN_CODE)
        .conn()
        .execute_one(
            r#"update
    flow_inst
  set
    rel_flow_model_id = flow_res.model_id
    from 
    (
      select
        flow_inst.id,
        flow_model2.id as model_id
      from
        flow_inst
        left join flow_model as flow_model1 on flow_inst.rel_flow_model_id = flow_model1.id
        inner join flow_model as flow_model2 on flow_inst.own_paths = flow_model2.own_paths
        and flow_model1.tag = flow_model2.tag
      WHERE
        flow_inst.own_paths <> flow_model1.own_paths
        and flow_model1.tag not in ('TICKET', 'PROJ')
    ) as flow_res
  where
    flow_inst.id = flow_res.id"#,
            vec![],
        )
        .await?;

    Ok(())
}

pub async fn init_flow_model(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let ticket_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["TICKET".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if ticket_init_model.is_none() {
        // 工单模板初始化
        FlowModelServ::init_model(
            "TICKET",
            vec![
                ("待处理", FlowSysStateKind::Start, ""),
                ("处理中", FlowSysStateKind::Progress, ""),
                ("待确认", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
                ("已撤销", FlowSysStateKind::Finish, ""),
            ],
            "待处理-处理中-待确认-已关闭-已撤销",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "处理中".to_string(),
                    name: "立即处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "已撤销".to_string(),
                    name: "撤销".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已撤销？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "待确认".to_string(),
                    name: "处理完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待确认？".to_string()),
                    }),
                    guard_by_his_operators: Some(true),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "确认解决".into(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "处理中".to_string(),
                    name: "未解决".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let req_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["REQ".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if req_init_model.is_none() {
        // 需求模板初始化
        FlowModelServ::init_model(
            "REQ",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Finish, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".into(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let product_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["PROJ".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if product_init_model.is_none() {
        FlowModelServ::init_model(
            "PROJ",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("存在风险", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
                ("已归档", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-存在风险-已完成-已关闭-已归档",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已归档".to_string(),
                    name: "归档".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已归档？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "已归档".to_string(),
                    name: "归档".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已归档？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已归档".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已归档".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let iter_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["ITER".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if iter_init_model.is_none() {
        FlowModelServ::init_model(
            "ITER",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("存在风险", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let task_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["TASK".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if task_init_model.is_none() {
        FlowModelServ::init_model(
            "TASK",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("存在风险", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该任务存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }

    Ok(())
}
