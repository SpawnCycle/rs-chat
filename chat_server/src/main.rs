mod logging;
mod routes;
mod types;
mod ws_fairing;
mod ws_handler;

use rocket::{Build, Rocket};

use crate::ws_fairing::WsFairing;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    logging::setup()?;

    rocket()
        .launch()
        .await
        .inspect_err(|err| log::error!("There was a fatal error: {err}"))?;

    Ok(())
}

fn rocket() -> Rocket<Build> {
    rocket::build().attach(WsFairing)
}
