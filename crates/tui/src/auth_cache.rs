use std::fs::File;

use anyhow::{anyhow, Context, Result};
use bblearn_api::{AuthState, Client, Credentials};
use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

/// Caches credentials and authentication state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCache {
    pub creds: Credentials,
    auth_state: AuthState,
}

const PREFIX: &str = "learn-tui";
const FILE_NAME: &str = "auth_cache.json";

impl AuthCache {
    pub fn from_client(client: &Client) -> Self {
        Self {
            auth_state: client.auth_state(),
            creds: client.creds.clone(),
        }
    }

    pub fn into_client(self) -> Result<Client> {
        Ok(Client::with_auth_state(self.creds, self.auth_state).unwrap())
    }

    pub fn load() -> Result<Self> {
        let path = BaseDirectories::with_prefix(PREFIX)?
            .find_config_file(FILE_NAME)
            .ok_or_else(|| anyhow!("auth cache does not exist"))?;

        let file = File::open(path).context("error opening auth cache")?;
        let config = serde_json::from_reader(&file).context("error deserialising auth cache")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = BaseDirectories::with_prefix(PREFIX)?.place_config_file(FILE_NAME)?;
        let mut file = File::create(path).context("error opening auth cache")?;

        serde_json::to_writer(&mut file, &self).context("error deserialising auth cache")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct LoginDetails {
    pub creds: Credentials,
    pub remember: bool,
}
