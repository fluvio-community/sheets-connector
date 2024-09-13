use fluvio_connector_common::{connector, secret::SecretString};

#[derive(Debug)]
#[connector(config, name = "sheet")]
pub(crate) struct SheetConfig {
    pub google_private_key: SecretString,
    pub google_client_email: SecretString,
    pub google_token_url: SecretString,
}
