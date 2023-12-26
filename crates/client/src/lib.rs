mod auth;

use std::time::SystemTime;

pub use auth::{Credentials, Error as AuthError, Password};
use thiserror::Error;
use ureq::{Agent, AgentBuilder};

pub const LEARN_BASE: &str = "https://www.learn.ed.ac.uk/";

/// A client, for using the blackboard learn API
pub struct Client {
    creds: Credentials,
    auth_expire: SystemTime,
    http: Agent,
}

#[derive(Error, Debug)]
pub enum ReqError {
    #[error("error authenticating: {}", .0)]
    AuthError(#[from] AuthError),

    #[error("http error: {}", .0)]
    HTTPError(#[from] ureq::Error),

    #[error("io error: {}", .0)]
    IOError(#[from] std::io::Error),
}

impl Client {
    pub fn new(creds: Credentials) -> Self {
        let http = AgentBuilder::new().redirects(10).build();

        Client {
            creds,
            http,
            auth_expire: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn ensure_auth(&self) -> Result<(), AuthError> {
        if self.auth_expire > SystemTime::now() {
            Ok(())
        } else {
            self.authenticate()
        }
    }

    // TODO: test
    pub fn health(&self) -> Result<serde_json::Value, ReqError> {
        self.ensure_auth()?;

        Ok(self
            .http
            .get(&format!("{}institution/api/health", LEARN_BASE))
            .call()?
            .into_json()?)
    }
}
