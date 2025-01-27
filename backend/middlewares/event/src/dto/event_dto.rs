use std::num::NonZeroU32;

use asteroid_mq::prelude::{TopicCode, TopicConfig, TopicOverflowConfig, TopicOverflowPolicy};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm::{self, FromQueryResult},
    serde_json::Value,
    web::poem_openapi,
};
pub(crate) fn format_code(code: &impl std::fmt::Display) -> String {
    format!("event/topic/{}", code)
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct EventTopicConfig {
    pub topic_code: String,
    #[oai(default)]
    pub blocking: bool,
    pub overflow_policy: Option<String>,
    #[oai(default)]
    pub overflow_size: i32,
    #[oai(default)]
    pub check_auth: bool,
}

impl EventTopicConfig {
    pub fn into_rbum_req(self) -> EventTopicAddOrModifyReq {
        EventTopicAddOrModifyReq {
            code: format_code(&self.topic_code),
            name: format_code(&self.topic_code),
            blocking: self.blocking,
            topic_code: self.topic_code,
            overflow_policy: self.overflow_policy.unwrap_or("RejectNew".to_string()),
            overflow_size: self.overflow_size.clamp(1, i32::MAX),
            check_auth: self.check_auth,
        }
    }
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, FromQueryResult)]
pub struct EventTopicAddOrModifyReq {
    pub code: String,
    pub name: String,
    #[oai(default)]
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
    #[oai(default)]
    pub check_auth: bool,
}

impl EventTopicAddOrModifyReq {
    pub fn into_topic_config(self) -> TopicConfig {
        TopicConfig {
            code: TopicCode::new(self.topic_code),
            blocking: self.blocking,
            overflow_config: Some(TopicOverflowConfig {
                policy: match self.overflow_policy.as_str() {
                    "RejectNew" => TopicOverflowPolicy::RejectNew,
                    "DropOld" => TopicOverflowPolicy::DropOld,
                    _ => TopicOverflowPolicy::default(),
                },
                size: NonZeroU32::new(self.overflow_size.clamp(1, u32::MAX as i32) as u32).expect("clamped"),
            }),
        }
    }
    pub fn from_config(config: TopicConfig) -> Self {
        Self {
            code: format_code(&config.code),
            name: format_code(&config.code),
            blocking: config.blocking,
            topic_code: config.code.to_string(),
            overflow_policy: config.overflow_config.as_ref().map_or("RejectNew".to_string(), |c| match c.policy {
                TopicOverflowPolicy::RejectNew => "RejectNew".to_string(),
                TopicOverflowPolicy::DropOld => "DropOld".to_string(),
            }),
            overflow_size: config.overflow_config.as_ref().map_or(0, |c| c.size.get() as i32),
            check_auth: false,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, FromQueryResult)]
pub struct EventTopicInfoResp {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub code: String,
    pub name: String,
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
    pub check_auth: bool,
}

impl EventTopicInfoResp {
    pub fn into_topic_config(self) -> TopicConfig {
        TopicConfig {
            code: TopicCode::new(self.topic_code),
            blocking: self.blocking,
            overflow_config: Some(TopicOverflowConfig {
                policy: match self.overflow_policy.as_str() {
                    "RejectNew" => TopicOverflowPolicy::RejectNew,
                    "DropOld" => TopicOverflowPolicy::DropOld,
                    _ => TopicOverflowPolicy::default(),
                },
                size: NonZeroU32::new(self.overflow_size.clamp(1, i32::MAX) as u32).expect("clamped"),
            }),
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct EventTopicFilterReq {
    pub basic: RbumBasicFilterReq,
    pub topic_code: Option<String>,
}

impl RbumItemFilterFetcher for EventTopicFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }

    fn rel(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }

    fn rel2(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventListenerRegisterReq {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub topic_code: TrimString,
    pub topic_sk: Option<String>,
    // #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub events: Option<Vec<TrimString>>,
    pub avatars: Vec<TrimString>,
    pub subscribe_mode: bool,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct EventRegisterResp {
    pub node_id: String,
    pub expire_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventListenerInfo {
    pub topic_code: String,
    pub subscribe_mode: bool,
    pub events: Option<Vec<String>>,
    pub avatars: Vec<String>,
    pub mgr: bool,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessageMgrWrap {
    pub msg: Value,
    pub ori_from_avatar: String,
    pub ori_to_avatars: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct TopicAuth {
    pub topic: String,
    pub ak: String,
    pub read: bool,
    pub write: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct SetTopicAuth {
    pub topic: String,
    pub read: bool,
    pub write: bool,
}
