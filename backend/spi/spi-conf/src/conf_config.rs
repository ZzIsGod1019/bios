use bios_basic::rbum::rbum_config::RbumConfig;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tardis::consts::{IP_LOCALHOST, IP_UNSPECIFIED};

use crate::dto::conf_auth_dto::RegisterRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ConfConfig {
    pub rbum: RbumConfig,
    /// token ttl in second, default as 18000
    pub token_ttl: u32,
    /// this should be specified when we have multi nodes.
    pub auth_key: String,
    pub auth_username: String,
    pub auth_password: String,
    pub nacos_port: u16,
    pub nacos_grpc_port: u16,
    pub nacos_host: IpAddr,
    pub placeholder_white_list: Vec<IpNet>,
    pub iam_client: IamClientConfig,
    pub data_id_env_config: String,
    pub group_env_config: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct IamClientConfig {
    pub base_url: String,
}

impl ConfConfig {
    pub(crate) fn get_admin_account(&self) -> RegisterRequest {
        RegisterRequest {
            username: Some(self.auth_username.clone().into()),
            password: Some(self.auth_password.clone().into()),
        }
    }
}
impl Default for ConfConfig {
    fn default() -> Self {
        use tardis::crypto::*;
        use tardis::rand::*;
        let auth_key = crypto_base64::TardisCryptoBase64.encode(random::<[u8; 32]>());
        let password = format!("{:016x}", random::<u128>());
        Self {
            // 18000 secs (5 hours)
            token_ttl: 18000,
            auth_key,
            auth_username: String::from("nacos"),
            auth_password: password,
            rbum: Default::default(),
            nacos_port: 8848,
            nacos_grpc_port: 9848,
            nacos_host: IP_UNSPECIFIED,
            placeholder_white_list: vec![IpNet::from(IP_LOCALHOST)],
            iam_client: Default::default(),
            data_id_env_config: ".env".to_string(),
            group_env_config: "DEFAULT-GROUP".to_string(),
        }
    }
}
