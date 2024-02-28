use std::{env, fs::{remove_file, File, create_dir_all}};

use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use edlearn_client::{AuthState, Client, Credentials};
use serde::{Deserialize, Serialize};

/// Caches credentials and authentication state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCache {
    pub creds: Credentials,
    auth_state: AuthState,
}

const FILE_NAME: &str = "learn-tui.json";

impl AuthCache {
    /// Retrieve the state from a client
    pub fn from_client(client: &Client) -> Self {
        Self {
            auth_state: client.auth_state(),
            creds: client.creds.clone(),
        }
    }

    /// Get a client using this state
    pub fn into_client(self) -> Result<Client> {
        Ok(Client::with_auth_state(self.creds, self.auth_state).unwrap())
    }

    /// Clear the authentication cache, if it exists
    pub fn clear() -> Result<()> {
        let Ok(path) = state_file_location() else {
            return Ok(()); // already cleared
        };

        remove_file(path)?;

        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = state_file_location()?;
        let file = File::open(path).context("error opening auth cache")?;
        let config = serde_json::from_reader(&file).context("error deserialising auth cache")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = state_file_location()?;
        create_dir_all(path.parent().unwrap())?;
        let mut file = File::create(path).context("error opening auth cache")?;

        serde_json::to_writer(&mut file, &self).context("error deserialising auth cache")?;

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
fn state_file_location() -> Result<Utf8PathBuf> {
    let mut out = if let Ok(loc) = env::var("XDG_STATE_DIR") {
        Utf8PathBuf::from(loc)
    } else {
        // Ok here, since this isn't compiled on windows.
        #[allow(deprecated)]
        let mut home = env::home_dir().ok_or_else(|| anyhow!("user home dir not set"))?;
        home.push(".local");
        home.push(".state");
        home.try_into().expect("non utf8 path")
    };
    
    out.push(FILE_NAME);

    Ok(out)
}

#[cfg(target_os = "windows")]
fn state_file_location() -> Result<Utf8PathBuf> {
    let mut out = if let Ok(loc) = env::var("LOCALAPPDATA") {
        Utf8PathBuf::from(loc)
    } else {
        // This method is deprecated because if you're using a *nix environment emulator like cygwin, it will return a unix-style path
        // instead of the user's real, windows, home dir.
        // Personally, I think this is fine - if a user wants to emulate a *nix environment then we should behave like one.
        // Most likely LOCALAPPDATA will be set, so this isn't super important anyway.
        #[allow(deprecated)]
        let mut home = env::home_dir().ok_or_else(|| anyhow!("user home dir not set"))?;
        home.push("AppData");
        home.push("Local");
        home.try_into().expect("non utf8 path")
    };
    
    out.push(FILE_NAME);

    Ok(out)
}

/// A user's login preferences
#[derive(Debug)]
pub struct LoginDetails {
    pub creds: Credentials,
    pub remember: bool,
}
