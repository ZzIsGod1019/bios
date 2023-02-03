use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::ptr::replace;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::spi::dto::spi_bs_dto::SpiBsCertResp;

use super::serv::spi_bs_serv::SpiBsServ;
use super::spi_constants;

pub trait SpiTardisFunInstExtractor {
    fn tardis_fun_inst(&self) -> TardisFunsInst;
}

#[cfg(feature = "default")]
impl SpiTardisFunInstExtractor for tardis::web::poem::Request {
    fn tardis_fun_inst(&self) -> TardisFunsInst {
        let serv_domain = self.original_uri().path().split('/').collect::<Vec<&str>>()[1];
        TardisFuns::inst_with_db_conn(serv_domain.to_string(), None)
    }
}

pub struct SpiBsInst {
    pub client: Box<dyn Any + Send>,
    pub ext: HashMap<String, String>,
}

impl SpiBsInst {
    pub fn inst<T>(&'static self) -> (&'static T, &'static HashMap<String, String>, String) {
        let c = self.client.as_ref().downcast_ref::<T>().unwrap();
        (c, &self.ext, self.kind_code())
    }

    pub fn kind_code(&self) -> String {
        self.ext.get(spi_constants::SPI_KIND_CODE_FLAG).unwrap().to_string()
    }
}

static mut SPI_BS_CACHES: Option<HashMap<String, SpiBsInst>> = None;

#[async_trait]
pub trait SpiBsInstExtractor {
    async fn init<'a, F, T>(&self, ctx: &'a TardisContext, mgr: bool, init_funs: F) -> TardisResult<String>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send;

    async fn bs<'a>(&self, ctx: &'a TardisContext) -> TardisResult<&'static SpiBsInst>;

    async fn init_bs<'a, F, T>(&self, ctx: &'a TardisContext, mgr: bool, init_funs: F) -> TardisResult<&'static SpiBsInst>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send;

    fn bs_not_implemented(&self, bs_code: &str) -> TardisError;
}

#[async_trait]
impl SpiBsInstExtractor for TardisFunsInst {
    /// Initialize the backend service instance
    ///
    /// # Arguments
    ///
    /// * `ctx` - Request Context
    /// * `mgr` - Whether it is a managed request
    /// * `init_fun` - The initialization function called when the backend service instance is not initialized
    ///
    /// # Return
    ///
    /// the backend service instance kind
    /// ```
    async fn init<'a, F, T>(&self, ctx: &'a TardisContext, mgr: bool, init_fun: F) -> TardisResult<String>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send,
    {
        let cache_key = format!("{}-{}", self.module_code(), ctx.owner);
        unsafe {
            if SPI_BS_CACHES.is_none() {
                replace(&mut SPI_BS_CACHES, Some(HashMap::new()));
            }
            match &mut SPI_BS_CACHES {
                None => panic!("[SPI] CACHE instance doesn't exist"),
                Some(caches) => {
                    if !caches.contains_key(&cache_key) {
                        let spi_bs = SpiBsServ::get_bs_by_rel(&ctx.owner, self, ctx).await?;
                        info!(
                            "[SPI] Init and cache backend service instance [{}]:{}",
                            cache_key.clone(),
                            TardisFuns::json.obj_to_string(&spi_bs)?
                        );
                        let kind_code = spi_bs.kind_code.clone();
                        let mut spi_bs_inst = init_fun(spi_bs, ctx, mgr).await?;
                        spi_bs_inst.ext.insert(spi_constants::SPI_KIND_CODE_FLAG.to_string(), kind_code);
                        caches.insert(cache_key.clone(), spi_bs_inst);
                    }
                    Ok(caches.get(&cache_key).unwrap().kind_code())
                }
            }
        }
    }

    /// Fetch the backend service instance
    ///
    /// # Arguments
    ///
    /// * `ctx` - Request Context
    ///
    /// # Return
    ///
    /// the backend service instance
    /// ```
    async fn bs<'a>(&self, ctx: &'a TardisContext) -> TardisResult<&'static SpiBsInst> {
        let cache_key = format!("{}-{}", self.module_code(), ctx.owner);
        unsafe {
            match &mut SPI_BS_CACHES {
                None => panic!("[SPI] CACHE instance doesn't exist"),
                Some(caches) => Ok(caches.get(&cache_key).unwrap()),
            }
        }
    }

    /// Initialize the backend service instance and fetch it
    ///
    /// # Arguments
    ///
    /// * `ctx` - Request Context
    /// * `mgr` - Whether it is a managed request
    /// * `init_fun` - The initialization function called when the backend service instance is not initialized
    ///
    /// # Return
    ///
    /// the backend service instance
    /// ```
    async fn init_bs<'a, F, T>(&self, ctx: &'a TardisContext, mgr: bool, init_fun: F) -> TardisResult<&'static SpiBsInst>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send,
    {
        self.init(ctx, mgr, init_fun).await?;
        self.bs(ctx).await
    }

    fn bs_not_implemented(&self, bs_code: &str) -> TardisError {
        bs_not_implemented(bs_code)
    }
}

pub fn bs_not_implemented(bs_code: &str) -> TardisError {
    TardisError::not_implemented(
        &format!("Backend service kind {} does not exist or SPI feature is not enabled", bs_code),
        "406-rbum-*-enum-init-error",
    )
}
