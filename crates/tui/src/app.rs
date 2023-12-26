use anyhow::Result;
use bblearn_api::Client;
use std::env;

use crate::config::Config;

pub struct App {
    pub running: bool,
    client: Client,
}
impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Result<Self> {
        let client = match Config::load() {
            Ok(c) => Client::with_auth_state(c.creds, c.auth_state).unwrap(),
            Err(e) => {
                println!("error loading config: {:?}", e);

                let creds = (
                    env::var("LEARN_USERNAME").unwrap(),
                    env::var("LEARN_PASSWORD").unwrap().into(),
                );
                Client::new(creds)
            }
        };

        Ok(Self {
            running: true,
            client,
        })
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        Config::from_client(&self.client).save().unwrap();
        self.running = false;
    }
}
