use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCsAccountAttrApi;

/// System Console Account Attr API
/// 系统控制台账号属性API
///
/// Note: the current account attr only supports tenant level.
/// 注意：当前账号属性仅支持租户级别。
#[poem_openapi::OpenApi(prefix_path = "/cs/account/attr", tag = "bios_basic::ApiTag::System")]
impl IamCsAccountAttrApi {
    /// Add Account Attr By Tenant Id
    /// 添加账号属性
    #[oai(path = "/", method = "post")]
    async fn add_attr(&self, tenant_id: Query<Option<String>>, add_req: Json<IamKindAttrAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr(&add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Account Attr By Account Attr Id
    /// 修改账号属性
    #[oai(path = "/:id", method = "put")]
    async fn modify_attr(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        mut modify_req: Json<RbumKindAttrModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::modify_account_attr(&id.0, &mut modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account Attr By Account Attr Id
    /// 获取账号属性
    #[oai(path = "/:id", method = "get")]
    async fn get_attr(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumKindAttrDetailResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::get_account_attr(&id.0, true, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account Attrs By Tenant Id
    /// 查找账号属性
    #[oai(path = "/", method = "get")]
    async fn find_attrs(&self, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attrs(&funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Account Attr By Account Attr Id
    /// 删除账号属性
    #[oai(path = "/:id", method = "delete")]
    async fn delete_attr(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::delete_account_attr(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Account Ext Attr Values By Account Id
    /// 查找账号属性值
    #[oai(path = "/value", method = "get")]
    async fn find_account_attr_values(
        &self,
        account_id: Query<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<HashMap<String, String>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attr_values(&account_id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
