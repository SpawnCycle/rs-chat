mod config;
mod logging;
mod types;
mod ws;

use rocket::{Build, Rocket};

use crate::ws::WsFairing;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    logging::setup()?;

    rocket()
        .launch()
        .await
        .inspect_err(|err| log::error!("There was a fatal error during: {err}"))?;

    Ok(())
}

fn rocket() -> Rocket<Build> {
    rocket::custom(config::rocket()).attach(WsFairing)
}
