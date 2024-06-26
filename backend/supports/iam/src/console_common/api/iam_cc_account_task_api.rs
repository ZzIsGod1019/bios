use bios_basic::{helper::request_helper::try_set_real_ip_from_req_to_ctx, process::task_processor::TaskProcessor};
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    console_common::serv::{iam_cc_account_task_serv::IamCcAccountTaskServ, iam_cc_role_task_serv::IamCcRoleTaskServ},
    iam_constants,
};

#[derive(Clone, Default)]
pub struct IamCcAccountTaskApi;

/// Common Console Account task API
#[poem_openapi::OpenApi(prefix_path = "/cc/account/task", tag = "bios_basic::ApiTag::Common")]
impl IamCcAccountTaskApi {
    #[oai(path = "/", method = "get")]
    async fn execute_account_task(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        IamCcAccountTaskServ::execute_account_task(&funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    #[oai(path = "/search", method = "get")]
    async fn execute_account_search_task(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        IamCcAccountTaskServ::execute_account_search_task(&funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    #[oai(path = "/role", method = "get")]
    async fn execute_role_task(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        IamCcRoleTaskServ::execute_role_task(&funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
