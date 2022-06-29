use std::collections::HashMap;
use std::time::Duration;

use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;
use bios_iam::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountBoneResp};
use bios_iam::basic::dto::iam_app_dto::IamAppAggAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfInfo;
use bios_iam::basic::dto::iam_role_dto::IamRoleBoneResp;
use bios_iam::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetItemWithDefaultSetAddReq};
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggAddReq;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_TENANT;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(sysadmin_name: &str, sysadmin_password: &str, client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【test_iam_scenes_common】");

    client.login(sysadmin_name, sysadmin_password, None, None, None, true).await?;

    // Add Tenant
    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamTenantAggAddReq {
                name: TrimString("测试公司1".to_string()),
                icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                contact_phone: None,
                note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("admin".to_string()),
                admin_password: Some("123456".to_string()),
                cert_conf_by_user_pwd: IamUserPwdCertConfInfo {
                    ak_rule_len_min: 2,
                    ak_rule_len_max: 20,
                    sk_rule_len_min: 2,
                    sk_rule_len_max: 20,
                    sk_rule_need_num: false,
                    sk_rule_need_uppercase: false,
                    sk_rule_need_lowercase: false,
                    sk_rule_need_spec_char: false,
                    sk_lock_cycle_sec: 60,
                    sk_lock_err_times: 2,
                    sk_lock_duration_sec: 60,
                    repeatable: false,
                    expire_sec: 6000,
                },
                cert_conf_by_phone_vcode: false,
                cert_conf_by_mail_vcode: true,
                disabled: None,
            },
        )
        .await;
    sleep(Duration::from_secs(1)).await;
    let account_id = client.login("admin", "123456", Some(tenant_id.clone()), None, None, true).await?.account_id;
    let cate_node_id: String = client
        .post(
            "/ct/org/cate",
            &IamSetCateAddReq {
                name: TrimString("综合服务中心".to_string()),
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                bus_code: None,
                icon: None,
                sort: None,
                ext: None,
                rbum_parent_cate_id: None,
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/org/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: cate_node_id.to_string(),
                sort: 0,
                rel_rbum_item_id: account_id.clone(),
            },
        )
        .await;

    common_console_by_tenant(client).await?;

    // Add Account
    let app_account_id: String = client
        .post(
            "/ct/account",
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("devops应用管理员".to_string()),
                cert_user_name: TrimString("user_dp".to_string()),
                cert_password: TrimString("123456".to_string()),
                cert_phone: None,
                cert_mail: Some(TrimString("devopsxxx@xx.com".to_string())),
                role_ids: None,
                org_node_ids: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00002".to_string())]),
            },
        )
        .await;
    let _: String = client
        .put(
            "/ct/org/item",
            &IamSetItemWithDefaultSetAddReq {
                set_cate_id: cate_node_id.to_string(),
                sort: 0,
                rel_rbum_item_id: app_account_id.clone(),
            },
        )
        .await;

    // Add App
    let app_id: String = client
        .post(
            "/ct/app",
            &IamAppAggAddReq {
                app_name: TrimString("devops project".to_string()),
                app_icon: None,
                app_sort: None,
                app_contact_phone: None,
                admin_id: app_account_id.clone(),
                disabled: None,
            },
        )
        .await;
    client.login("user_dp", "123456", Some(tenant_id.clone()), Some(app_id), None, true).await?;

    common_console_by_app(client).await?;

    Ok(())
}

pub async fn common_console_by_tenant(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【common_console_by_tenant】");

    // Find Accounts
    let accounts: TardisPage<IamAccountBoneResp> = client.get("/cc/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    assert!(accounts.records.iter().any(|i| i.name == "测试管理员"));

    // Find Account Name By Ids
    let accounts: Vec<String> = client.get(&format!("/cc/account/name?ids={}", accounts.records[0].id)).await;
    assert_eq!(accounts[0], "测试管理员");

    // Find Roles
    let roles: TardisPage<IamRoleBoneResp> = client.get("/cc/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 1);
    assert!(roles.records.iter().any(|i| i.name == "tenant_admin"));

    // Find Org Cates By Current Tenant
    let res_tree: Vec<RbumSetTreeResp> = client.get("/cc/org/tree").await;
    assert_eq!(res_tree.len(), 1);
    assert!(res_tree.iter().any(|i| i.name == "综合服务中心"));
    assert_eq!(res_tree[0].rbum_set_items.len(), 1);
    assert_eq!(res_tree[0].rbum_set_items[0].rel_rbum_item_name, "测试管理员");

    Ok(())
}

pub async fn common_console_by_app(client: &mut BIOSWebTestClient) -> TardisResult<()> {
    info!("【common_console_by_app】");

    // Find Accounts
    let accounts: TardisPage<IamAccountBoneResp> = client.get("/cc/account?name=devops&page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 1);
    assert!(accounts.records.iter().any(|i| i.name == "devops应用管理员"));
    let accounts: TardisPage<IamAccountBoneResp> = client.get("/cc/account?page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);
    assert!(accounts.records.iter().any(|i| i.name == "devops应用管理员"));

    // Find Account Name By Ids
    let accounts: Vec<String> = client.get(&format!("/cc/account/name?ids={};{}", accounts.records[0].id, accounts.records[1].id)).await;
    assert_eq!(accounts.len(), 2);
    assert!(accounts.contains(&"devops应用管理员".to_string()));

    // Find Roles
    let roles: TardisPage<IamRoleBoneResp> = client.get("/cc/role?page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 2);
    assert!(roles.records.iter().any(|i| i.name == "app_admin"));
    let roles: TardisPage<IamRoleBoneResp> = client.get("/cc/role?name=app&page_number=1&page_size=10").await;
    assert_eq!(roles.total_size, 1);
    assert!(roles.records.iter().any(|i| i.name == "app_admin"));

    // Find Org Cates By Current Tenant
    let res_tree: Vec<RbumSetTreeResp> = client.get("/cc/org/tree").await;
    assert_eq!(res_tree.len(), 1);
    assert!(res_tree.iter().any(|i| i.name == "综合服务中心"));
    assert_eq!(res_tree[0].rbum_set_items.len(), 2);
    assert!(res_tree[0].rbum_set_items.iter().any(|i| i.rel_rbum_item_name == "测试管理员"));
    assert!(res_tree[0].rbum_set_items.iter().any(|i| i.rel_rbum_item_name == "devops应用管理员"));

    Ok(())
}