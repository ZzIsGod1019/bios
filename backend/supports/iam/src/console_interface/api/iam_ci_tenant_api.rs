use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeNodeResp;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::web::Query;
use tardis::web::poem_openapi;

use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::basic::dto::iam_tenant_dto::IamTenantBoneResp;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_enumeration::IamSetKind;
use crate::{
    basic::{
        dto::{iam_filer_dto::IamTenantFilterReq, iam_tenant_dto::IamTenantAggDetailResp},
        serv::iam_tenant_serv::IamTenantServ,
    },
    iam_constants,
};

#[derive(Clone, Default)]
pub struct IamCiTenantApi;

/// # Interface Console Tenant API
/// 接口控制台租户API
#[poem_openapi::OpenApi(prefix_path = "/ci/tenant", tag = "bios_basic::ApiTag::Tenant")]
impl IamCiTenantApi {
    /// Find Tenants
    /// 查找租户
    #[oai(path = "/all", method = "get")]
    async fn find(&self) -> TardisApiResult<Vec<IamTenantBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::find_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &IamCertServ::get_anonymous_context(),
        )
        .await?;
        let result = result
            .into_iter()
            .map(|i| IamTenantBoneResp {
                id: i.id,
                name: i.name,
                icon: i.icon,
            })
            .collect();
        TardisResp::ok(result)
    }

    /// Get Current Tenant
    /// 获取当前租户
    #[oai(path = "/", method = "get")]
    async fn get(&self, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantAggDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Org Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    ///
    /// 通过当前租户查找组织树
    ///
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级。当树太大时，可以逐级查询
    #[oai(path = "/orgs", method = "get")]
    async fn get_orgs(
        &self,
        parent_sys_code: Query<Option<String>>,
        set_id: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetTreeNodeResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result.main)
    }
}
