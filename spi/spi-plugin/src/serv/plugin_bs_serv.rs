use bios_basic::{
    rbum::{
        dto::{
            rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumRelFilterReq},
            rbum_rel_agg_dto::RbumRelAggResp,
            rbum_rel_attr_dto::RbumRelAttrAddReq,
        },
        rbum_enumeration::RbumRelFromKind,
        serv::{
            rbum_crud_serv::RbumCrudOperation,
            rbum_item_serv::RbumItemCrudOperation,
            rbum_rel_serv::{RbumRelAttrServ, RbumRelServ},
        },
    },
    spi::{
        dto::spi_bs_dto::SpiBsFilterReq,
        serv::spi_bs_serv::SpiBsServ,
        spi_constants::{self, SPI_IDENT_REL_TAG},
    },
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::plugin_bs_dto::{PluginBsAddReq, PluginBsCertInfoResp, PluginBsInfoResp};

pub struct PluginBsServ;

impl PluginBsServ {
    pub async fn add_or_modify_plugin_rel_agg(bs_id: &str, app_tenant_id: &str, add_req: &mut PluginBsAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if !ctx.own_paths.contains(app_tenant_id) {
            return Err(funs.err().unauthorized(
                "spi_bs",
                "add_or_modify_plugin_rel_agg",
                &format!("plugin binding rel unauthorized {}.{} by {}", bs_id, app_tenant_id, ctx.own_paths),
                "401-plugin-ownership-illegal",
            ));
        }
        let bs = SpiBsServ::peek_item(bs_id, &SpiBsFilterReq::default(), funs, ctx).await?;
        if SpiBsServ::count_items(
            &SpiBsFilterReq {
                basic: RbumBasicFilterReq { ..Default::default() },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: true,
                    tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(app_tenant_id.to_owned()),
                    ..Default::default()
                }),
                kind_id: Some(bs.kind_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            > 0
        {
            let rel_agg = Self::get_bs_rel_agg(bs_id, app_tenant_id, funs, ctx).await?;
            for attrs in rel_agg.attrs {
                RbumRelAttrServ::delete_rbum(&attrs.id, funs, ctx).await?;
            }
            if let Some(attrs) = add_req.attrs.clone() {
                // todo check attrs
                for attr in &attrs {
                    RbumRelAttrServ::add_rbum(
                        &mut RbumRelAttrAddReq {
                            is_from: attr.is_from,
                            value: attr.value.to_string(),
                            name: attr.name.to_string(),
                            rel_rbum_rel_id: rel_agg.rel.id.to_string(),
                            rel_rbum_kind_attr_id: attr.rel_rbum_kind_attr_id.to_string(),
                            record_only: attr.record_only,
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                }
            }
        } else {
            SpiBsServ::add_rel_agg(bs.id.as_str(), app_tenant_id, add_req.attrs.clone(), None, funs, ctx).await?;
        }
        Ok(bs.id)
    }

    pub async fn paginate_bs_rel_agg(
        app_tenant_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<PluginBsInfoResp>> {
        let rel_agg = RbumRelServ::paginate_to_rels(
            spi_constants::SPI_IDENT_REL_TAG,
            app_tenant_id,
            page_number,
            page_size,
            desc_by_create,
            desc_by_update,
            funs,
            ctx,
        )
        .await?;
        let mut bs_records = vec![];
        for rel_agg in rel_agg.records {
            let bs = SpiBsServ::peek_item(&rel_agg.rel.from_rbum_id, &SpiBsFilterReq::default(), funs, ctx).await?;
            bs_records.push(PluginBsInfoResp {
                id: bs.id,
                name: bs.name,
                kind_id: bs.kind_id,
                kind_code: bs.kind_code,
                kind_name: bs.kind_name,
                rel: Some(rel_agg),
            });
        }
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size: rel_agg.total_size,
            records: bs_records,
        })
    }

    pub async fn get_bs(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsInfoResp> {
        let rel_agg = Self::get_bs_rel_agg(bs_id, app_tenant_id, funs, ctx).await?;
        let bs = SpiBsServ::peek_item(bs_id, &SpiBsFilterReq::default(), funs, ctx).await?;
        Ok(PluginBsInfoResp {
            id: bs.id,
            name: bs.name,
            kind_id: bs.kind_id,
            kind_code: bs.kind_code,
            kind_name: bs.kind_name,
            rel: Some(rel_agg),
        })
    }

    pub async fn get_bs_by_rel(kind_code: Option<String>, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsCertInfoResp> {
        let bs = SpiBsServ::find_one_item(
            &SpiBsFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: true,
                    tag: Some(SPI_IDENT_REL_TAG.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(app_tenant_id.to_string()),
                    ..Default::default()
                }),
                kind_code,
                domain_code: Some(funs.module_code().to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel", "not found backend service", "404-spi-bs-not-exist"))?;
        let rel_agg = Self::get_bs_rel_agg(bs.id.as_str(), app_tenant_id, funs, ctx).await?;
        Ok(PluginBsCertInfoResp {
            id: bs.id,
            name: bs.name,
            conn_uri: bs.conn_uri,
            ak: bs.ak,
            sk: bs.sk,
            ext: bs.ext,
            private: bs.private,
            rel: Some(rel_agg),
        })
    }

    pub async fn get_bs_by_rel_up(kind_code: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsCertInfoResp> {
        let own_paths = Self::get_parent_own_paths(ctx.own_paths.as_str())?;
        let mut resp = Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist"));
        for own_path in own_paths {
            resp = Self::get_bs_by_rel(kind_code.clone(), own_path.as_str(), funs, ctx).await;
            info!("【get_bs_by_rel_up】 {}: {}", own_path, resp.is_ok());
            if resp.is_ok() {
                break;
            }
        }
        match resp {
            Ok(bs) => Ok(bs),
            Err(err) => Err(err),
        }
    }

    pub async fn get_bs_rel_agg(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumRelAggResp> {
        let rel_agg = RbumRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(bs_id.to_owned()),
                to_rbum_item_id: Some(app_tenant_id.to_owned()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_agg.len() > 1 || rel_agg.is_empty() {
            return Err(funs.err().conflict(&SpiBsServ::get_obj_name(), "get_bs", "not found backend", ""));
        }
        Ok(rel_agg.into_iter().next().unwrap())
    }

    pub fn get_parent_own_paths(own_paths: &str) -> TardisResult<Vec<String>> {
        if own_paths.is_empty() {
            return Ok(vec!["".to_string()]);
        }
        let mut paths = own_paths.split('/').map(|s| s.to_string()).collect::<Vec<String>>();
        paths.reverse();
        paths.push("".to_string());
        Ok(paths)
    }
}