use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;
use tardis::{
    basic::error::TardisError,
    log,
    web::{
        poem::{self, endpoint::BoxEndpoint, http::HeaderValue, Body, Endpoint, IntoResponse, Middleware, Request, Response},
        web_resp::TardisResp,
        web_server::BoxMiddleware,
    },
};

use crate::{iam_config::IamConfig, iam_constants};
#[derive(Clone, Debug)]
pub struct EncryptMW;

impl EncryptMW {
    pub fn boxed() -> BoxMiddleware<'static> {
        Box::new(EncryptMW)
    }
}

impl Middleware<BoxEndpoint<'static>> for EncryptMW {
    type Output = BoxEndpoint<'static>;

    fn transform(&self, ep: BoxEndpoint<'static>) -> Self::Output {
        Box::new(poem::endpoint::ToDynEndpoint(EncryptMWImpl(ep)))
    }
}

pub struct EncryptMWImpl<E>(E);

impl<E: Endpoint> Endpoint for EncryptMWImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        let funs = iam_constants::get_tardis_inst();
        let _method = req.method().to_string();
        let _url = req.uri().to_string();
        let req_head_crypto_value = req.header(&funs.conf::<IamConfig>().crypto_conf.head_key_crypto).map(|v| v.to_string());
        let resp = self.0.call(req).await;
        match resp {
            Ok(resp) => {
                let mut resp = resp.into_response();

                //如果有这个头，那么需要返回加密
                if let Some(key_crypto) = req_head_crypto_value {
                    log::trace!("[Iam.Middleware] key_crypto:{key_crypto}");
                    let resp_body = resp.take_body().into_string().await?;

                    let mut headers = HashMap::new();
                    headers.insert(funs.conf::<IamConfig>().crypto_conf.head_key_crypto.to_string(), key_crypto);
                    let auth_encrypt_req = AuthEncryptReq { headers, body: resp_body.clone() };

                    let encrypt_resp: TardisResp<AuthEncryptResp> = funs
                        .web_client()
                        .put(&format!("{}/auth/crypto", funs.conf::<IamConfig>().crypto_conf.auth_url), &auth_encrypt_req, None)
                        .await
                        .map_err(|e| TardisError::internal_error(&format!("[Iam.Middleware] Encrypted api call error: {e}"), "500-auth-resp-crypto-error"))?
                        .body
                        .ok_or_else(|| TardisError::internal_error("[Iam.Middleware] Encrypted api call error: not found body", "500-auth-resp-crypto-error"))?;

                    if encrypt_resp.code != *"200" {
                        return Err(TardisError::internal_error(
                            &format!("[Iam.Middleware] Encrypted api call return error:{}", encrypt_resp.msg),
                            "500-auth-resp-crypto-error",
                        )
                        .into());
                    } else if let Some(resp_body) = encrypt_resp.data {
                        if let Some(encrypt_resp_header_value) = resp_body.headers.get(&funs.conf::<IamConfig>().crypto_conf.head_key_crypto) {
                            let resp_headers = resp.headers_mut();
                            if let Ok(header_value) = HeaderValue::from_str(encrypt_resp_header_value) {
                                resp_headers.insert(funs.conf::<IamConfig>().crypto_conf.get_crypto_header_name()?, header_value);
                            }
                            resp.set_body(Body::from_string(resp_body.body));
                        }
                    } else {
                        resp.set_body(Body::from_string(resp_body));
                    }
                }
                Ok(resp)
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthEncryptReq {
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AuthEncryptResp {
    pub headers: HashMap<String, String>,
    pub body: String,
}
