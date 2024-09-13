mod config;
mod sink;
use std::time::Duration;

use anyhow::anyhow;
use config::SheetConfig;
use futures::{SinkExt, StreamExt};

use fluvio_connector_common::{
    connector,
    consumer::ConsumerStream,
    future::retry::ExponentialBackoff,
    tracing::{error, trace, warn},
    Result, Sink,
};
use serde::{Deserialize, Serialize};
use sink::SheetsSink;

const BACKOFF_MIN: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(3600 * 24);

// Data payload structure to be inserted into the Google Sheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub range: String,
    pub values: Vec<Vec<serde_json::Value>>,
    pub major_dimension: String,
    pub spreadsheet_id: String,
}

#[connector(sink)]
async fn start(config: SheetConfig, mut stream: impl ConsumerStream) -> Result<()> {
    println!("Starting sheet-connector sink connector with {config:?}");
    let mut backoff = backoff_init()?;
    loop {
        let Some(wait) = backoff.next() else {
            // not currently possible, but if backoff strategy is changed later
            // this could kick in
            let msg = "Retry backoff exhausted";
            error!(msg);
            return Err(anyhow!(msg));
        };
        if wait >= BACKOFF_MAX {
            // max retry set to 24hrs
            error!("Max retry reached");
            continue;
        }
        let sink = SheetsSink::new(&config)?;
        let mut sink = match sink.connect(None).await {
            Ok(sink) => sink,
            Err(err) => {
                warn!(
                    "Error connecting to sink: \"{}\", reconnecting in {}.",
                    err,
                    humantime::format_duration(wait)
                );
                async_std::task::sleep(wait).await;
                continue; // loop and retry
            }
        };
        // reset the backoff on successful connect
        backoff = backoff_init()?;

        while let Some(item) = stream.next().await {
            let out: std::result::Result<Payload, serde_json::Error> =
                serde_json::from_slice(item?.as_ref());
            match out {
                Ok(payload) => {
                    trace!(?payload);
                    sink.send(payload).await?;
                }
                Err(_) => error!("data parsing error. try again"),
            }
        }
    }
}

fn backoff_init() -> Result<ExponentialBackoff> {
    let bmin: u64 = BACKOFF_MIN.as_millis().try_into()?;
    Ok(ExponentialBackoff::from_millis(bmin).max_delay(BACKOFF_MAX))
}
