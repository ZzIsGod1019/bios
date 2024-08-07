use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::IamRoleBoneResp;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRoleKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::Request;

#[derive(Clone, Default)]
pub struct IamCcRoleApi;

/// Common Console Role API
/// 通用控制台角色API
#[poem_openapi::OpenApi(prefix_path = "/cc/role", tag = "bios_basic::ApiTag::Common")]
impl IamCcRoleApi {
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
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamRoleBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    enabled: Some(true),
                    // scope_level: if IamAppServ::is_app_level_by_ctx(&ctx.0) { Some(RBUM_SCOPE_LEVEL_APP) } else { None },
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
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result
                .records
                .into_iter()
                .map(|item| IamRoleBoneResp {
                    id: item.id,
                    name: item.name,
                    code: item.code,
                    kind: item.kind,
                    scope_level: item.scope_level,
                    icon: item.icon,
                    in_base: item.in_base,
                    in_embed: item.in_embed,
                    extend_role_id: item.extend_role_id,
                })
                .collect(),
        })
    }

    /// Find pub Rel Res By Role Id
    /// 根据角色ID查找公开关联资源
    #[oai(path = "/:id/pub_res", method = "get")]
    async fn find_rel_res_with_pub(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut ctx = ctx.0;
        ctx.own_paths = "".to_string();
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::find_simple_rels(
            &id.0,
            desc_by_create.0,
            desc_by_update.0,
            Some(vec![
                RbumScopeLevelKind::Root.to_int(),
                RbumScopeLevelKind::L1.to_int(),
                RbumScopeLevelKind::L2.to_int(),
                RbumScopeLevelKind::L3.to_int(),
            ]),
            Some(vec![
                RbumScopeLevelKind::Private.to_int(),
                RbumScopeLevelKind::Root.to_int(),
                RbumScopeLevelKind::L1.to_int(),
                RbumScopeLevelKind::L2.to_int(),
                RbumScopeLevelKind::L3.to_int(),
            ]),
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Role Name By Ids
    /// 根据角色ID查找角色名称
    ///
    /// Return format: ["<id>,<name>"]
    /// 返回格式：["<id>,<name>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // Role Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamRoleServ::find_name_by_ids(ids, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Embed Subrole Id
    /// 获取嵌套子角色ID
    #[oai(path = "/get_embed_subrole_id", method = "get")]
    async fn get_embed_subrole_id(&self, id: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::get_embed_sub_role_id(&id, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
