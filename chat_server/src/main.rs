mod logging;

use rocket::{Build, Config, Rocket};

use chat_server::ws::WsFairing;

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
    rocket::custom(rocket_cfg()).attach(WsFairing::new("/api/ws".into()))
}

// TODO: Actually implement file configs and args
#[must_use]
pub fn rocket_cfg() -> Config {
    Config {
        cli_colors: false,
        ..Default::default()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn server_can_launch() -> Result<(), anyhow::Error> {
        rocket().ignite().await?;

        Ok(())
    }
}
