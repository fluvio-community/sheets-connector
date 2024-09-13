use crate::{config::SheetConfig, Payload};
use async_trait::async_trait;
use fluvio::Offset;
use fluvio_connector_common::{LocalBoxSink, Result, Sink};
use google_sheets4::{
    api::ValueRange,
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2::{self, ServiceAccountKey},
    Error, Sheets,
};

pub(crate) struct SheetsSink {
    secret: oauth2::ServiceAccountKey,
}

impl SheetsSink {
    pub(crate) fn new(config: &SheetConfig) -> Result<Self> {
        let private_key = config.google_private_key.resolve()?;
        let client_email = config.google_client_email.resolve()?;
        let token_uri = config.google_token_url.resolve()?;
        let secret: oauth2::ServiceAccountKey = ServiceAccountKey {
            client_email,
            private_key,
            token_uri,
            auth_provider_x509_cert_url: None,
            auth_uri: None,
            client_id: None,
            client_x509_cert_url: None,
            key_type: None,
            private_key_id: None,
            project_id: None,
        };

        Ok(Self { secret })
    }
}

// Function to get OAuth2 access token using environment variables for credentials

#[async_trait]
impl Sink<Payload> for SheetsSink {
    async fn connect(self, _offset: Option<Offset>) -> Result<LocalBoxSink<Payload>> {
        let auth = oauth2::ServiceAccountAuthenticator::builder(self.secret)
            .build()
            .await?;
        let hub = Sheets::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .unwrap()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );

        let unfold = futures::sink::unfold(
            hub,
            |hub: Sheets<HttpsConnector<HttpConnector>>, record: Payload| async move {
                let req = ValueRange {
                    values: Some(record.values),
                    range: None,
                    major_dimension: None,
                };
                let result = hub
                    .spreadsheets()
                    .values_append(req, &record.spreadsheet_id, &record.range)
                    .value_input_option("USER_ENTERED")
                    .insert_data_option("OVERWRITE")
                    .include_values_in_response(false)
                    .doit()
                    .await;
                match result {
                    Err(e) => match e {
                        // The Error enum provides details about what exactly happened.
                        // You can also just use its `Debug`, `Display` or `Error` traits
                        Error::HttpError(_)
                        | Error::Io(_)
                        | Error::MissingAPIKey
                        | Error::MissingToken(_)
                        | Error::Cancelled
                        | Error::UploadSizeLimitExceeded(_, _)
                        | Error::Failure(_)
                        | Error::BadRequest(_)
                        | Error::FieldClash(_)
                        | Error::JsonDecodeError(_, _) => println!("{}", e),
                    },
                    Ok(res) => println!("Success: {:?}", res.0.status()),
                }
                Ok::<_, anyhow::Error>(hub)
            },
        );
        Ok(Box::pin(unfold))
    }
}
