mod auth;

pub use auth::{AuthState, Credentials, Error as AuthError, Password};
use cookie_store::CookieStore;
use serde::Deserialize;
use thiserror::Error;
use ureq::{Agent, AgentBuilder};

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
    HTTPError(#[from] ureq::Error),

    #[error("io error: {}", .0)]
    IOError(#[from] std::io::Error),
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
            Err(Error::HTTPError(ureq::Error::Status(c, _))) if c / 100 == 4 => {
                self.authenticate()?;
                f()
            }
            x => x,
        }
    }

    /// Call server health endpoint
    pub fn health(&self) -> Result<HealthResp, Error> {
        self.with_reattempt_auth(|| {
            Ok(self
                .http
                .get(&format!("{}institution/api/health", LEARN_BASE))
                .call()?
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
