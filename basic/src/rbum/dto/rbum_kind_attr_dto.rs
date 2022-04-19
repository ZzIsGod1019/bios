use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAttrAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub sort: Option<u32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_id: String,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAttrModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub label: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub sort: Option<u32>,
    pub main_column: Option<bool>,
    pub position: Option<bool>,
    pub capacity: Option<bool>,
    pub overload: Option<bool>,
    pub data_type: Option<RbumDataTypeKind>,
    pub widget_type: Option<RbumWidgetTypeKind>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub default_value: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub options: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub action: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindAttrSummaryResp {
    pub id: String,
    pub name: String,
    pub label: String,
    pub sort: u32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindAttrDetailResp {
    pub id: String,
    pub name: String,
    pub label: String,
    pub note: String,
    pub sort: u32,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub data_type: RbumDataTypeKind,
    pub widget_type: RbumWidgetTypeKind,
    pub default_value: String,
    pub options: String,
    pub required: bool,
    pub min_length: u32,
    pub max_length: u32,
    pub action: String,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_kind_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}