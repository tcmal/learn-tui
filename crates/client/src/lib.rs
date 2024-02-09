mod auth;
pub mod content;
pub mod course;
pub mod membership;
pub mod terms;
pub mod users;

use std::sync::Arc;

pub use auth::{AuthState, Credentials, Error as AuthError, Password};
use log::debug;
use reqwest::blocking::{Client as HTTPClient, ClientBuilder as HTTPClientBuilder, Response};
use reqwest_cookie_store::{CookieStore, CookieStoreRwLock};
use serde::Deserialize;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub const LEARN_BASE: &str = "https://www.learn.ed.ac.uk/";

/// A client, for using the blackboard learn API
pub struct Client {
    pub creds: Credentials,
    http: HTTPClient,
    cookies: Arc<CookieStoreRwLock>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error authenticating: {}", .0)]
    AuthError(#[from] AuthError),

    #[error("http error: {}", .0)]
    HTTPError(#[from] reqwest::Error),

    #[error("io error: {}", .0)]
    IOError(#[from] std::io::Error),

    #[error("serde error: {}", .0)]
    SerdeError(#[from] serde_json::Error),

    #[error("content leaf was malformed")]
    BadContentLeaf,
}

impl Client {
    pub fn new(creds: Credentials) -> Self {
        let cookies = Arc::new(CookieStoreRwLock::new(CookieStore::new(None)));
        let http = HTTPClientBuilder::new()
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        Client {
            creds,
            http,
            cookies,
        }
    }

    pub fn with_auth_state(
        creds: Credentials,
        state: AuthState,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let cookies = Arc::new(CookieStoreRwLock::new(CookieStore::load_json(
            state.0.as_slice(),
        )?));
        let http = HTTPClientBuilder::new()
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        Ok(Self {
            creds,
            http,
            cookies,
        })
    }

    /// Clone the current client, returning a new one.
    /// The two clients will share the same authentication state.
    pub fn clone_sharing_state(&self) -> Self {
        Self {
            creds: self.creds.clone(),
            http: self.http.clone(),
            cookies: self.cookies.clone(),
        }
    }

    /// Get the underlying HTTP client.
    pub fn http(&self) -> &HTTPClient {
        &self.http
    }

    /// Wrapper for attempting a request, and re-trying if it fails for authentication reasons
    pub fn with_reattempt_auth<T, F>(&self, mut f: F) -> Result<T, Error>
    where
        F: FnMut() -> Result<T, Error>,
    {
        match f() {
            Err(Error::HTTPError(e)) => {
                debug!("http error: {e}");
                if e.status().filter(|c| c.as_u16() / 100 == 4).is_some() {
                    self.authenticate()?;
                    f()
                } else {
                    Err(Error::HTTPError(e))
                }
            }
            x => x,
        }
    }

    pub(crate) fn get<T: for<'a> Deserialize<'a>>(&self, url: &str) -> Result<T, Error> {
        self.with_reattempt_auth(|| {
            let resp = self
                .http
                .get(format!("{}{}", LEARN_BASE, url))
                .send()
                .and_then(Response::error_for_status)?
                .error_for_status()?;
            if log::log_enabled!(log::Level::Debug) {
                let s = resp.text()?;
                debug!("response: {}", s);
                Ok(serde_json::from_str(&s)?)
            } else {
                Ok(resp.json()?)
            }
        })
    }

    /// Call server health endpoint
    pub fn health(&self) -> Result<HealthResp, Error> {
        self.with_reattempt_auth(|| {
            Ok(self
                .http
                .get(format!("{}institution/api/health", LEARN_BASE))
                .send()
                .and_then(Response::error_for_status)?
                .json()?)
        })
    }
}

/// Response given by the health endpoint API
#[derive(Debug, Deserialize, Clone)]
pub struct HealthResp {
    pub version: String,
    pub status: String,
    pub migration: String,
}
