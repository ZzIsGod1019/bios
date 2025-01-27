use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;

use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq, IamRoleAggCopyReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleSummaryResp};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRoleKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
use tardis::{log, tokio};
#[derive(Clone, Default)]
pub struct IamCsRoleApi;

/// System Console Role API
/// 系统控制台角色API
#[poem_openapi::OpenApi(prefix_path = "/cs/role", tag = "bios_basic::ApiTag::System")]
impl IamCsRoleApi {
    /// Add Role
    /// 添加角色
    #[oai(path = "/", method = "post")]
    async fn add(&self, tenant_id: Query<Option<String>>, mut add_req: Json<IamRoleAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        add_req.0.role.kind = Some(IamRoleKind::System);
        let result = IamRoleServ::add_role_agg(&mut add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// copy Role
    /// 复制角色
    #[oai(path = "/copy", method = "post")]
    async fn copy(&self, mut copy_req: Json<IamRoleAggCopyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        copy_req.0.role.kind = Some(IamRoleKind::System);
        let result = IamRoleServ::copy_role_agg(&mut copy_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(task_id)
        } else {
            TardisResp::ok(result)
        }
    }

    /// Modify Role By Role Id
    /// 根据角色ID修改角色
    ///
    /// When code = 202, the return value is the asynchronous task id
    /// 当 code = 202 时，返回值为异步任务id
    #[oai(path = "/:id", method = "put")]
    async fn modify(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        mut modify_req: Json<IamRoleAggModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<String>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::modify_role_agg(&id.0, &mut modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Get Role By Role Id
    /// 根据角色ID获取角色
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamRoleDetailResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::get_item(&id.0, &IamRoleFilterReq::default(), &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    /// 查找角色
    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        kind: Query<Option<IamRoleKind>>,
        in_base: Query<Option<bool>>,
        in_embed: Query<Option<bool>>,
        extend_role_id: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                kind: kind.0,
                in_base: in_base.0,
                in_embed: in_embed.0,
                extend_role_id: extend_role_id.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Role By Role Id
    /// 根据角色ID删除角色
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_item_with_all_rels(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Add Role Rel Account
    /// 添加角色关联账号
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(
        &self,
        id: Path<String>,
        account_id: Path<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_account(&id.0, &account_id.0, None, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch Add Role Rel Account
    /// 批量添加角色关联账号
    #[oai(path = "/:id/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_account(
        &self,
        id: Path<String>,
        account_ids: Path<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(account_ids.0.split(',').map(|account_id| async { IamRoleServ::add_rel_account(&id.0, account_id, None, &funs, &ctx).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Account
    /// 删除角色关联账号
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(
        &self,
        id: Path<String>,
        account_id: Path<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_account(&id.0, &account_id.0, None, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Role Rel Account
    /// 批量删除角色关联账号
    #[oai(path = "/:id/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_account(
        &self,
        id: Path<String>,
        account_ids: Path<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamRoleServ::delete_rel_account(&id.0, s, None, &funs, &ctx).await?;
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Accounts By Role Id
    /// 根据角色ID统计关联账号数量
    #[oai(path = "/:id/account/total", method = "get")]
    async fn count_rel_accounts(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_accounts(&id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Role Rel Res
    /// 添加角色关联资源
    #[oai(path = "/:id/res/:res_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_res(&id.0, &res_id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Res
    /// 删除角色关联资源
    #[oai(path = "/:id/res/:res_id", method = "delete")]
    async fn delete_rel_res(
        &self,
        id: Path<String>,
        res_id: Path<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_res(&id.0, &res_id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Res By Role Id
    /// 根据角色ID统计关联资源数量
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_res(&id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Rel Res By Role Id
    /// 根据角色ID查找关联资源
    #[oai(path = "/:id/res", method = "get")]
    async fn find_rel_res(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let ctx = if let Some(tenant_id) = tenant_id.0 {
            IamCertServ::try_use_tenant_ctx(ctx.0, Some(tenant_id))?
        } else {
            IamCertServ::use_sys_ctx_unsafe(ctx.0)?
        };
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::find_simple_rel_res(&id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// add base embed role
    /// 添加基础嵌入角色
    #[oai(path = "/add_base_embed_role", method = "post")]
    async fn add_base_embed_role(&self, mut add_req: Json<IamRoleAddReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        tokio::spawn(async move {
            funs.begin().await.unwrap();
            add_req.0.in_embed = Some(true);
            match IamRoleServ::add_base_embed_role(&add_req.0, &funs, &ctx.0).await {
                Ok(_) => {
                    log::trace!("[Iam.Cs] add log success")
                }
                Err(e) => {
                    log::warn!("[Iam.Cs] failed to add log:{e}")
                }
            }
            funs.commit().await.unwrap();
        });

        TardisResp::ok(Void {})
    }
}