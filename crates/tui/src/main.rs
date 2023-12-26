use std::env;

use anyhow::Result;
use bblearn_api::Client;

fn main() -> Result<()> {
    env_logger::init();

    let creds = (
        env::var("LEARN_USERNAME").unwrap(),
        env::var("LEARN_PASSWORD").unwrap().into(),
    );
    let client = Client::new(creds);
    dbg!(client.health()?);

    Ok(())
}
