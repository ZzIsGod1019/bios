use std::collections::{HashMap, HashSet};

use bios_basic::rbum::{
    dto::rbum_filer_dto::RbumBasicFilterReq,
    serv::rbum_item_serv::RbumItemCrudOperation,
};
use bios_sdk_invoke::{
    clients::{
        base_spi_client::BaseSpiClient,
        iam_client::IamClient,
    },
    invoke_enumeration::InvokeModuleKind,
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::{
    dto::flow_model_dto::FlowModelFilterReq,
    serv::flow_model_serv::FlowModelServ,
};

pub struct FlowIamClient;

impl FlowIamClient {
    /// 透传调用 [`bios_sdk_invoke::clients::iam_client::IamClient::get_embed_subrole_id`]。
    pub async fn get_embed_subrole_id(role_ids: &Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let base_url = BaseSpiClient::module_url(InvokeModuleKind::Iam, funs).await?;
        let client = IamClient::new(ctx.owner.as_str(), funs, ctx, None, base_url.as_str());
        client.get_embed_subrole_id(&role_ids.join(",")).await
    }
    /// 透传调用 [`bios_sdk_invoke::clients::iam_client::IamClient::batch_get_embed_sub_role_by_own_paths`]。
    pub async fn batch_get_embed_sub_role_by_own_paths(
        role_id: &str,
        own_paths: &Vec<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<HashMap<String, String>> {
        let base_url = BaseSpiClient::module_url(InvokeModuleKind::Iam, funs).await?;
        let client = IamClient::new(ctx.owner.as_str(), funs, ctx, None, base_url.as_str());
        client.batch_get_embed_sub_role_by_own_paths(role_id, own_paths).await
    }
}
