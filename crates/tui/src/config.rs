use std::fs::File;

use anyhow::{anyhow, Context, Result};
use bblearn_api::{AuthState, Client, Credentials};
use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub creds: Credentials,
    pub auth_state: AuthState,
}

impl Config {
    pub fn from_client(client: Client) -> Self {
        Self {
            auth_state: client.auth_state(),
            creds: client.creds,
        }
    }

    pub fn load() -> Result<Self> {
        let path = BaseDirectories::with_prefix("learn-tui")?
            .find_config_file("config.json")
            .ok_or_else(|| anyhow!("config does not exist"))?;

        let file = File::open(&path).context("error opening config file")?;
        let config = serde_json::from_reader(&file).context("error deserialising config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = BaseDirectories::with_prefix("learn-tui")?.place_config_file("config.json")?;

        let mut file = File::create(&path).context("error opening config file")?;
        serde_json::to_writer(&mut file, &self).context("error deserialising config file")?;

        Ok(())
    }
}
