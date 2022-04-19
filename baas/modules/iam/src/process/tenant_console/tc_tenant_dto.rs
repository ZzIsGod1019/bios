/*
 * Copyright 2022. the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use validator::Validate;

use crate::process::basic_dto::AccountIdentKind;

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantModifyReq {
    // 租户名称
    #[validate(length(min = 2, max = 255))]
    pub name: Option<TrimString>,
    // 租户图标
    #[validate(length(min = 2, max = 1000))]
    pub icon: Option<String>,
    // 是否开放账号注册
    pub allow_account_register: Option<bool>,
    // 租户扩展信息，Json格式
    #[validate(length(min = 2, max = 5000))]
    pub parameters: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct TenantDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 租户名称
    #[validate(length(max = 255))]
    pub name: TrimString,
    // 租户图标
    #[validate(length(max = 1000))]
    pub icon: String,
    // 是否开放账号注册
    pub allow_account_register: bool,
    // 租户扩展信息，Json格式
    #[validate(length(max = 5000))]
    pub parameters: String,
    // 租户状态
    #[validate(length(max = 255))]
    pub status: String,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantCertAddReq {
    // 凭证类型名称
    #[validate(length(min = 2, max = 255), regex = "bios::basic::field::R_CODE_CS")]
    pub category: String,
    // 凭证保留的版本数量
    pub version: i32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantCertModifyReq {
    // 凭证保留的版本数量
    pub version: Option<i32>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct TenantCertDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 凭证类型名称
    #[validate(length(max = 255))]
    pub category: String,
    // 凭证保留的版本数量
    pub version: i32,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantIdentAddReq {
    // 租户认证类型名称
    pub kind: AccountIdentKind,
    // 认证AK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule_note: Option<String>,
    // 认证AK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule: Option<String>,
    // 认证SK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule_note: Option<String>,
    // 认证SK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule: Option<String>,
    // 认证有效时间（秒）
    pub valid_time: i32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TenantIdentModifyReq {
    // 认证AK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule_note: Option<String>,
    // 认证AK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_ak_rule: Option<String>,
    // 认证SK校验正则规则说明
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule_note: Option<String>,
    // 认证SK校验正则规则
    #[validate(length(min = 2, max = 2000))]
    pub valid_sk_rule: Option<String>,
    // 认证有效时间（秒）
    pub valid_time: Option<i32>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct TenantIdentDetailResp {
    #[validate(length(max = 32))]
    pub id: String,
    // 租户认证类型名称
    #[validate(length(max = 255))]
    pub kind: String,
    // 认证AK校验正则规则说明
    #[validate(length(max = 2000))]
    pub valid_ak_rule_note: String,
    // 认证AK校验正则规则
    #[validate(length(max = 2000))]
    pub valid_ak_rule: String,
    // 认证SK校验正则规则说明
    #[validate(length(max = 2000))]
    pub valid_sk_rule_note: String,
    // 认证SK校验正则规则
    #[validate(length(max = 2000))]
    pub valid_sk_rule: String,
    // 认证有效时间（秒）
    pub valid_time: i32,
    #[validate(length(max = 255))]
    pub create_user: String,
    #[validate(length(max = 255))]
    pub update_user: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}