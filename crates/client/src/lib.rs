mod auth;
pub mod content;
pub mod course;
pub mod membership;
pub mod terms;
pub mod users;

pub use auth::{AuthState, Credentials, Error as AuthError, Password};
use cookie_store::CookieStore;
use log::debug;
use serde::Deserialize;
use thiserror::Error;
use ureq::{Agent, AgentBuilder};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub const LEARN_BASE: &str = "https://www.learn.ed.ac.uk/";

/// A client, for using the blackboard learn API
pub struct Client {
    pub creds: Credentials,
    http: Agent,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error authenticating: {}", .0)]
    AuthError(#[from] AuthError),

    #[error("http error: {}", .0)]
    HTTPError(#[from] Box<ureq::Error>),

    #[error("io error: {}", .0)]
    IOError(#[from] std::io::Error),

    #[error("serde error: {}", .0)]
    SerdeError(#[from] serde_json::Error),

    #[error("content leaf was malformed")]
    BadContentLeaf,
}

impl Client {
    pub fn new(creds: Credentials) -> Self {
        let http = AgentBuilder::new().redirects(10).strict_mode(false).build();

        Client { creds, http }
    }

    pub fn with_auth_state(
        creds: Credentials,
        state: AuthState,
    ) -> Result<Self, cookie_store::Error> {
        let store = CookieStore::load_json(state.0.as_slice())?;
        let http = AgentBuilder::new()
            .redirects(10)
            .strict_mode(false)
            .cookie_store(store)
            .build();

        Ok(Self { creds, http })
    }

    /// Wrapper for attempting a request, and re-trying if it fails for authentication reasons
    pub fn with_reattempt_auth<T, F>(&self, mut f: F) -> Result<T, Error>
    where
        F: FnMut() -> Result<T, Error>,
    {
        match f() {
            Err(Error::HTTPError(e)) => match e.as_ref() {
                ureq::Error::Status(c, _) if c / 100 == 4 => {
                    self.authenticate()?;
                    f()
                }
                _ => Err(Error::HTTPError(e)),
            },
            x => x,
        }
    }

    pub(crate) fn get<T: for<'a> Deserialize<'a>>(&self, url: &str) -> Result<T, Error> {
        self.with_reattempt_auth(|| {
            let resp = self
                .http
                .get(&format!("{}{}", LEARN_BASE, url))
                .call()
                .map_err(Box::new)?;
            if log::log_enabled!(log::Level::Debug) {
                let s = resp.into_string()?;
                debug!("response: {}", s);
                Ok(serde_json::from_str(&s)?)
            } else {
                Ok(resp.into_json()?)
            }
        })
    }

    /// Call server health endpoint
    pub fn health(&self) -> Result<HealthResp, Error> {
        self.with_reattempt_auth(|| {
            Ok(self
                .http
                .get(&format!("{}institution/api/health", LEARN_BASE))
                .call()
                .map_err(Box::new)?
                .into_json()?)
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
