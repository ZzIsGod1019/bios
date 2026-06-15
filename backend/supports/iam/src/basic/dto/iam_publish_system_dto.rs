use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

/// 发布系统新增请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    /// 系统名称
    #[oai(validator(min_length = "1", max_length = "50"))]
    pub name: TrimString,
    /// 系统标识
    #[oai(validator(max_length = "50"))]
    pub sys_ident: Option<String>,
    /// 所属分公司（租户 ID）
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub rel_tenant_id: TrimString,
    pub scope_level: Option<RbumScopeLevelKind>,
    /// 描述
    #[oai(validator(max_length = "500"))]
    pub description: Option<String>,
}

/// 发布系统修改请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemModifyReq {
    /// 系统名称
    #[oai(validator(min_length = "1", max_length = "50"))]
    pub name: Option<TrimString>,
    /// 系统标识
    #[oai(validator(max_length = "50"))]
    pub sys_ident: Option<String>,
    /// 所属分公司（租户 ID）
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub rel_tenant_id: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    /// 描述
    #[oai(validator(max_length = "500"))]
    pub description: Option<String>,
}

/// 发布系统概要响应
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamPublishSystemSummaryResp {
    pub id: String,
    pub name: String,
    pub sys_ident: Option<String>,
    pub rel_tenant_id: String,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// 发布系统详情响应
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug, Clone)]
pub struct IamPublishSystemDetailResp {
    pub id: String,
    pub name: String,
    pub sys_ident: Option<String>,
    pub rel_tenant_id: String,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
