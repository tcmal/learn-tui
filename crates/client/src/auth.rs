//! Code for authenticating to Edinburgh University's Learn instance
//!
//! Thank you to @kilolympus and @chaives for figuring out the login process
//! See: https://git.tardisproject.uk/kilo/echo360-downloader
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Client;

/// Information used to login
pub type Credentials = (String, Password);

#[derive(Error, Debug)]
pub enum Error {
    #[error("we didn't login for some reason. check your credentials?")]
    LoginFailed,

    #[error("couldn't identify the SAMLRequest payload. text: {}", .0)]
    NoSAMLRequest(String),

    #[error("couldn't identify the SAMLResponse payload. text: {}", .0)]
    NoSAMLResponse(String),

    #[error("error communicating with learn: {}", .0)]
    LearnReqError(Box<ureq::Error>),

    #[error("error communicating with EASE: {}", .0)]
    EaseReqError(Box<ureq::Error>),

    #[error("error communicating with idp: {}", .0)]
    IDPReqError(Box<ureq::Error>),

    #[error("misc I/O error: {}", .0)]
    IOError(#[from] std::io::Error),
}

impl Client {
    pub(crate) fn authenticate(&self) -> Result<(), Error> {
        self.ease_login()?;
        self.learn_login()?;

        Ok(())
    }

    fn ease_login(&self) -> Result<(), Error> {
        // Get once to set the cookies
        self.http
            .get("https://www.ease.ed.ac.uk/")
            .call()
            .map_err(|e| Error::EaseReqError(Box::new(e)))?;

        // Login to CoSign
        let text = self
            .http
            .post("https://www.ease.ed.ac.uk/cosign.cgi")
            .send_form(&[
                ("login", &self.creds.0),
                ("password", self.creds.1.as_ref()),
            ])
            .map_err(|e| Error::EaseReqError(Box::new(e)))?
            .into_string()?;

        if !text.contains("/logout/logout.cgi") {
            return Err(Error::LoginFailed);
        }

        Ok(())
    }

    fn learn_login(&self) -> Result<(), Error> {
        // Initiates the login process
        const LEARN_LOGIN_URL: &str = "https://www.learn.ed.ac.uk/auth-saml/saml/login?apId=_175_1&redirectUrl=https%3A%2F%2Fwww.learn.ed.ac.uk%2Fultra";
        const SSO_SAML_URL: &str = "https://idp.ed.ac.uk/idp/profile/SAML2/POST/SSO";
        const LEARN_CALLBACK_URL: &str =
            "https://www.learn.ed.ac.uk/auth-saml/saml/SSO/alias/_175_1";
        let text = self
            .http
            .get(LEARN_LOGIN_URL)
            .call()
            .map_err(|e| Error::LearnReqError(Box::new(e)))?
            .into_string()?;

        let samlreq_re = Regex::new(r#"name="SAMLRequest" value="([^"]*)""#).unwrap();
        let Some(caps) = samlreq_re.captures(&text) else {
            return Err(Error::NoSAMLRequest(text));
        };
        let samlreq = &caps[1];

        // Authn Request
        let text = self
            .http
            .post(SSO_SAML_URL)
            .send_form(&[("SAMLRequest", samlreq)])
            .map_err(|e| Error::IDPReqError(Box::new(e)))?
            .into_string()?;
        let samlresp_re = Regex::new(r#"name="SAMLResponse" value="([^"]*)""#).unwrap();
        let Some(caps) = samlresp_re.captures(&text) else {
            return Err(Error::NoSAMLResponse(text));
        };
        let samlresp = &caps[1];

        self.http
            .post(LEARN_CALLBACK_URL)
            .send_form(&[("SAMLResponse", samlresp)])
            .map_err(|e| Error::LearnReqError(Box::new(e)))?;

        Ok(())
    }

    /// Serialise the auth state, for persistence
    pub fn auth_state(&self) -> AuthState {
        let mut ser = Vec::new();
        self.http
            .cookie_store()
            .save_incl_expired_and_nonpersistent_json(&mut ser)
            .unwrap();
        AuthState(ser)
    }
}

/// Contains cached authentication cookies
#[derive(Serialize, Deserialize, Clone)]
pub struct AuthState(pub(crate) Vec<u8>);

impl std::fmt::Debug for AuthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthState (***)")
    }
}

/// A password, wrapped so we don't print it by accident
#[derive(Clone, Serialize, Deserialize)]
pub struct Password(String);
impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Password (******)")
    }
}

impl From<String> for Password {
    fn from(value: String) -> Self {
        Password(value)
    }
}

impl From<Password> for String {
    fn from(val: Password) -> Self {
        val.0
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
