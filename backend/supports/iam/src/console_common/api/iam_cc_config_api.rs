use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdResp;

use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_platform_serv::IamPlatformServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcConfigApi;

/// Common Console Config API
/// 通用控制台配置API
#[poem_openapi::OpenApi(prefix_path = "/cc/config", tag = "bios_basic::ApiTag::Common")]
impl IamCcConfigApi {
    /// Get config
    /// 获取配置
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCertConfUserPwdResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = if IamAccountServ::is_global_account(&ctx.0.owner, &funs, &ctx.0).await? {
            let new_ctx = IamAccountServ::new_context_if_account_is_global(&ctx.0, &funs).await?;
            IamPlatformServ::get_platform_config_agg(&funs, &new_ctx).await?.cert_conf_by_user_pwd
        } else {
            IamTenantServ::get_tenant_config_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &funs, &ctx.0).await?.cert_conf_by_user_pwd
        };
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
