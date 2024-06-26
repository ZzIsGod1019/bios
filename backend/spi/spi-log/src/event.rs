use std::time::Duration;

use crate::{log_constants::EVENT_ADD_LOG, log_initializer::get_tardis_inst, serv};
use bios_sdk_invoke::clients::event_client::{self, EventTopicConfig};
use tardis::{
    basic::result::TardisResult,
    log::{error, info, warn},
    tokio,
    web::ws_processor::TardisWebsocketMessage,
    TardisFuns,
};
pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

pub async fn start_log_event_service(config: &EventTopicConfig) -> TardisResult<()> {
    info!("[BIOS.Log] starting event service");
    let funs = get_tardis_inst();
    let client = event_client::EventClient::new(config.base_url.as_str(), &funs);
    let mut event_conf = config.clone();
    if event_conf.avatars.is_empty() {
        event_conf.avatars.push(format!("{}/{}", event_conf.topic_code, env!("CARGO_PKG_NAME")))
    }
    let resp = client.register(&event_conf.into()).await?;
    let ws_client = TardisFuns::ws_client(&resp.ws_addr, |message| async move {
        if !message.is_text() {
            return None;
        }
        let Ok(json_str) = message.to_text() else {
            return None;
        };
        info!("[BIOS.Log] event msg: {json_str}");
        let Ok(TardisWebsocketMessage { msg, event, .. }) = TardisFuns::json.str_to_obj(json_str) else {
            return None;
        };
        match event.as_deref() {
            Some(EVENT_ADD_LOG) => {
                let Ok((mut req, ctx)) = TardisFuns::json.json_to_obj(msg) else {
                    return None;
                };
                tokio::spawn(async move {
                    let funs = get_tardis_inst();
                    let result = serv::log_item_serv::add(&mut req, &funs, &ctx).await;
                    if let Err(err) = result {
                        error!("[BIOS.Log] failed to log item: {}", err);
                    }
                });
            }
            Some(unknown_event) => {
                warn!("[BIOS.Log] event receive unknown event {unknown_event}")
            }
            _ => {}
        }
        None
    })
    .await?;
    tokio::spawn(async move {
        loop {
            // it's ok todo so, reconnect will be blocked until the previous ws_client is dropped
            let result = ws_client.reconnect().await;
            if let Err(err) = result {
                error!("[BIOS.Log] failed to reconnect to event service: {}", err);
            }
            tokio::time::sleep(RECONNECT_INTERVAL).await;
        }
    });
    Ok(())
}
