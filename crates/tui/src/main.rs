use std::env;

use anyhow::Result;
use bblearn_api::Client;

use crate::config::Config;

mod config;

fn main() -> Result<()> {
    env_logger::init();

    let client = init_client()?;

    dbg!(client.health()?);

    Config::from_client(client).save()?;

    Ok(())
}

fn init_client() -> Result<Client> {
    match Config::load() {
        Ok(c) => Ok(Client::with_auth_state(c.creds, c.auth_state).unwrap()),
        Err(e) => {
            println!("error loading config: {:?}", e);

            let creds = (
                env::var("LEARN_USERNAME").unwrap(),
                env::var("LEARN_PASSWORD").unwrap().into(),
            );
            Ok(Client::new(creds))
        }
    }
}
