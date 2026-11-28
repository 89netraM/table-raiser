use messages::{WakeEndpoint, WakeRequest};
use postcard_rpc::{
    header::VarSeqKind,
    host_client::{HostClient, HostErr},
    standard_icd::{ERROR_PATH, PingEndpoint, WireError},
};
use std::{convert::Infallible, time::Duration};
use thiserror::Error;

pub struct TableRaiserClient {
    pub client: HostClient<WireError>,
}

impl TableRaiserClient {
    pub async fn new() -> Self {
        let client = HostClient::new_raw_nusb(
            |d| d.product_string() == Some("Table Raiser"),
            ERROR_PATH,
            8,
            VarSeqKind::Seq2,
        )
        .await;
        Self { client }
    }

    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    pub async fn ping(&self, id: u32) -> Result<u32, ClientError<Infallible>> {
        let val = self.client.send_resp::<PingEndpoint>(&id).await?;
        Ok(val)
    }

    pub async fn wake(&self, time: Duration) -> Result<(), ClientError<Infallible>> {
        self.client
            .send_resp::<WakeEndpoint>(&WakeRequest(time.as_millis() as u64))
            .await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ClientError<E> {
    #[error("Communication error: {0}")]
    Comms(HostErr<WireError>),
    #[error("{0}")]
    Endpoint(E),
}

impl<E> From<HostErr<WireError>> for ClientError<E> {
    fn from(value: HostErr<WireError>) -> Self {
        Self::Comms(value)
    }
}

trait FlattenErr {
    type Good;
    type Bad;
    fn flatten(self) -> Result<Self::Good, ClientError<Self::Bad>>;
}

impl<T, E> FlattenErr for Result<T, E> {
    type Good = T;
    type Bad = E;
    fn flatten(self) -> Result<Self::Good, ClientError<Self::Bad>> {
        self.map_err(ClientError::Endpoint)
    }
}
