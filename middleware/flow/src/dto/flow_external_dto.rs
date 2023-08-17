use serde::{Deserialize, Serialize};
use serde_json::Value;
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalReq {
    pub kind: FlowExternalKind,
    pub curr_tag: String,
    pub curr_bus_obj_id: String,
    pub target_state: Option<String>,
    pub params: Vec<FlowExternalParams>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalKind {
    FetchRelObj,
    ModifyField,
    NotifyChanges,
}

#[derive(Debug, Deserialize, Serialize, poem_openapi::Union)]
pub enum FlowExternalParams {
    FetchRelObj(FlowExternalFetchRelObjReq),
    ModifyField(FlowExternalModifyFieldReq),
    NotifyChanges(FlowExternalNotifyChangesReq),
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjReq {
    pub rel_tag: String,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjResp {
    pub curr_tag: String,
    pub curr_bus_obj_id: String,
    pub rel_bus_objs: Vec<RelBusObjResp>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RelBusObjResp {
    pub rel_tag: String,
    pub rel_bus_obj_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldReq {
    pub var_id: Option<String>,
    pub var_name: Option<String>,
    pub value: Option<Value>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldResp {}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesReq {
    pub changed_vars: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesResp {}