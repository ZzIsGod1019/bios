use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::IamAccountAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamTokenCertConfAddReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::dto::iam_cert_dto::IamUserPwdCertAddReq;
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAddReq, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_enumeration::{IAMRelKind, IamCertTokenKind};

pub struct IamCsTenantServ;

impl<'a> IamCsTenantServ {
    pub async fn add_tenant(add_req: &mut IamCsTenantAddReq, funs: &TardisFunsInst<'a>, system_cxt: &TardisContext) -> TardisResult<(String, String)> {
        IamRoleServ::need_sys_admin(funs, system_cxt).await?;

        let tenant_admin_id = TardisFuns::field.nanoid();
        let tenant_id = IamTenantServ::get_new_id();
        let tenant_cxt = TardisContext {
            own_paths: tenant_id.clone(),
            ak: "".to_string(),
            token: "".to_string(),
            token_kind: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: tenant_admin_id.to_string(),
        };
        IamTenantServ::add_item(
            &mut IamTenantAddReq {
                id: Some(TrimString(tenant_id.clone())),
                name: add_req.tenant_name.clone(),
                icon: add_req.tenant_icon.clone(),
                sort: None,
                contact_phone: add_req.tenant_contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
            },
            funs,
            &tenant_cxt,
        )
        .await?;
        IamAccountServ::add_item(
            &mut IamAccountAddReq {
                id: Some(TrimString(tenant_admin_id.clone())),
                name: add_req.admin_name.clone(),
                icon: None,
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_TENANT,
            },
            funs,
            &tenant_cxt,
        )
        .await?;

        IamRelServ::add_rel(
            IAMRelKind::IamAccountRole,
            &tenant_admin_id,
            &IamBasicInfoManager::get().role_tenant_admin_id,
            None,
            None,
            funs,
            &tenant_cxt,
        )
        .await?;
        IamCertUserPwdServ::add_cert_conf(
            &mut IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None,
            },
            Some(tenant_id.to_string()),
            funs,
            &tenant_cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenDefault.to_string()),
                coexist_num: iam_constants::RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenDefault,
            Some(tenant_id.to_string()),
            funs,
            &tenant_cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPc.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPc,
            Some(tenant_id.to_string()),
            funs,
            &tenant_cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPhone.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPhone,
            Some(tenant_id.to_string()),
            funs,
            &tenant_cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPad.to_string()),
                coexist_num: 1,
                expire_sec: Some(iam_constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPad,
            Some(tenant_id.to_string()),
            funs,
            &tenant_cxt,
        )
        .await?;

        let pwd = IamCertServ::get_new_pwd();
        IamCertUserPwdServ::add_cert(
            &mut IamUserPwdCertAddReq {
                ak: TrimString(add_req.admin_username.0.to_string()),
                sk: TrimString(pwd.to_string()),
            },
            &tenant_admin_id,
            Some(&tenant_id),
            funs,
            &tenant_cxt,
        )
        .await?;
        Ok((tenant_id, pwd))
    }

    pub async fn modify_tenant(id: &str, modify_req: &mut IamCsTenantModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamTenantServ::modify_item(
            id,
            &mut IamTenantModifyReq {
                name: None,
                icon: None,
                sort: None,
                contact_phone: None,
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_tenant(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamTenantDetailResp> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamTenantServ::get_item(id, &IamTenantFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_tenants(
        q_id: Option<String>,
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamTenantSummaryResp>> {
        IamRoleServ::need_sys_admin(funs, cxt).await?;
        IamTenantServ::paginate_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ids: q_id.map(|id| vec![id]),
                    name: q_name,
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }
}