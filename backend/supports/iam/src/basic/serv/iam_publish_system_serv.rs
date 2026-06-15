use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::{dto::TardisContext, result::TardisResult};
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::TardisFunsInst;

use crate::basic::domain::iam_publish_system;
use crate::basic::dto::iam_filer_dto::IamPublishSystemFilterReq;
use crate::basic::dto::iam_publish_system_dto::{
    IamPublishSystemAddReq, IamPublishSystemDetailResp, IamPublishSystemModifyReq, IamPublishSystemSummaryResp,
};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::IamRelKind;

pub struct IamPublishSystemServ;

#[async_trait]
impl
    RbumItemCrudOperation<
        iam_publish_system::ActiveModel,
        IamPublishSystemAddReq,
        IamPublishSystemModifyReq,
        IamPublishSystemSummaryResp,
        IamPublishSystemDetailResp,
        IamPublishSystemFilterReq,
    > for IamPublishSystemServ
{
    fn get_ext_table_name() -> &'static str {
        iam_publish_system::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_publish_system_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn before_add_item(add_req: &mut IamPublishSystemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_tenant_exist(&add_req.rel_tenant_id, funs, ctx).await?;
        Self::check_name_duplicate(&add_req.name.to_string(), None, funs, ctx).await?;
        if let Some(sys_ident) = &add_req.sys_ident {
            if !sys_ident.is_empty() {
                Self::check_sys_ident_duplicate(sys_ident, None, funs, ctx).await?;
            }
        }
        Ok(())
    }

    async fn before_modify_item(id: &str, modify_req: &mut IamPublishSystemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(rel_tenant_id) = &modify_req.rel_tenant_id {
            Self::check_tenant_exist(rel_tenant_id, funs, ctx).await?;
            let item = Self::get_item(
                id,
                &IamPublishSystemFilterReq {
                    basic: RbumBasicFilterReq {
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if item.rel_tenant_id != rel_tenant_id.to_string() && Self::count_app_rel(id, funs, ctx).await? > 0 {
                return Err(funs.err().conflict(
                    &Self::get_obj_name(),
                    "modify",
                    "publish system is associated with apps and cannot change tenant",
                    "409-iam-publish-system-tenant-change-conflict",
                ));
            }
        }
        if let Some(name) = &modify_req.name {
            Self::check_name_duplicate(&name.to_string(), Some(id), funs, ctx).await?;
        }
        if let Some(sys_ident) = &modify_req.sys_ident {
            if !sys_ident.is_empty() {
                Self::check_sys_ident_duplicate(sys_ident, Some(id), funs, ctx).await?;
            }
        }
        Ok(())
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamPublishSystemDetailResp>> {
        if Self::count_app_rel(id, funs, ctx).await? > 0 {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                "publish system is associated with apps and cannot be deleted",
                "409-iam-publish-system-delete-conflict",
            ));
        }
        Ok(None)
    }

    async fn package_item_add(add_req: &IamPublishSystemAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamPublishSystemAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_publish_system::ActiveModel> {
        Ok(iam_publish_system::ActiveModel {
            id: Set(id.to_string()),
            sys_ident: Set(add_req.sys_ident.clone().filter(|s| !s.is_empty())),
            rel_tenant_id: Set(add_req.rel_tenant_id.to_string()),
            description: Set(add_req.description.clone()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamPublishSystemModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: None,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamPublishSystemModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_publish_system::ActiveModel>> {
        if modify_req.sys_ident.is_none() && modify_req.rel_tenant_id.is_none() && modify_req.description.is_none() {
            return Ok(None);
        }
        let mut model = iam_publish_system::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if modify_req.sys_ident.is_some() {
            model.sys_ident = Set(modify_req.sys_ident.clone().filter(|s| !s.is_empty()));
        }
        if let Some(rel_tenant_id) = &modify_req.rel_tenant_id {
            model.rel_tenant_id = Set(rel_tenant_id.to_string());
        }
        if modify_req.description.is_some() {
            model.description = Set(modify_req.description.clone());
        }
        Ok(Some(model))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamPublishSystemFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_publish_system::Entity, iam_publish_system::Column::SysIdent));
        query.column((iam_publish_system::Entity, iam_publish_system::Column::RelTenantId));
        query.column((iam_publish_system::Entity, iam_publish_system::Column::Description));
        if let Some(sys_ident) = &filter.sys_ident {
            query.and_where(Expr::col((iam_publish_system::Entity, iam_publish_system::Column::SysIdent)).eq(sys_ident.as_str()));
        }
        if let Some(rel_tenant_id) = &filter.rel_tenant_id {
            query.and_where(Expr::col((iam_publish_system::Entity, iam_publish_system::Column::RelTenantId)).eq(rel_tenant_id.as_str()));
        }
        Ok(())
    }
}

impl IamPublishSystemServ {
    /// 平台级系统名称重名校验
    async fn check_name_duplicate(name: &str, exclude_id: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(existing) = Self::find_one_item(
            &IamPublishSystemFilterReq {
                basic: RbumBasicFilterReq {
                    name: Some(name.to_string()),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            if exclude_id.map(|id| id != existing.id).unwrap_or(true) {
                return Err(funs.err().conflict(&Self::get_obj_name(), "check_name", "name already exists", "409-iam-publish-system-name-exist"));
            }
        }
        Ok(())
    }

    /// 平台级系统标识重名校验
    async fn check_sys_ident_duplicate(sys_ident: &str, exclude_id: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(existing) = Self::find_one_item(
            &IamPublishSystemFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sys_ident: Some(sys_ident.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        {
            if exclude_id.map(|id| id != existing.id).unwrap_or(true) {
                return Err(funs.err().conflict(
                    &Self::get_obj_name(),
                    "check_sys_ident",
                    "sys_ident already exists",
                    "409-iam-publish-system-sys-ident-exist",
                ));
            }
        }
        Ok(())
    }

    async fn check_tenant_exist(rel_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if IamTenantServ::count_items(
            &crate::basic::dto::iam_filer_dto::IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![rel_tenant_id.to_string()]),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            == 0
        {
            return Err(funs.err().not_found(&Self::get_obj_name(), "check_tenant", "tenant not found", "404-iam-publish-system-tenant-not-exist"));
        }
        Ok(())
    }

    async fn count_app_rel(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        IamRelServ::count_to_rels(&IamRelKind::IamAppPublishSystem, id, funs, &global_ctx).await
    }
}
