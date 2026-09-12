use chat_lib::Discovery;
use tokio::sync::mpsc::Sender;
use url::Url;

use crate::{consts::CLIENT, requests::room_discovery};

pub fn start_discovery(tx: Sender<AppTaskResult>, base_url: Url) {
    tokio::spawn(async {
        let tx = tx;
        let base_url = base_url;

        match room_discovery(&CLIENT, &base_url).await {
            Ok(val) => {
                let payload = AppTaskPayload::discovery(Ok(val));
                let _ = tx
                    .send(AppTaskResult {
                        base: base_url,
                        payload,
                    })
                    .await;
            }
            Err(err) => {
                let payload = AppTaskPayload::discovery(Err(err.into()));
                let _ = tx
                    .send(AppTaskResult {
                        base: base_url,
                        payload,
                    })
                    .await;
            }
        }
    });
}

#[derive(Debug)]
pub struct AppTaskResult {
    pub base: Url,
    pub payload: AppTaskPayload,
}

#[derive(Debug)]
pub enum AppTaskPayload {
    Discovery(anyhow::Result<Discovery>),
}

impl AppTaskPayload {
    pub fn discovery(dis: impl Into<anyhow::Result<Discovery>>) -> Self {
        Self::Discovery(dis.into())
    }
}

impl From<Discovery> for AppTaskPayload {
    fn from(value: Discovery) -> Self {
        Self::Discovery(Ok(value))
    }
}
