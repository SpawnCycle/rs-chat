use rocket::Config;

// TODO: Actually implement file configs and args
pub fn rocket() -> Config {
    Config {
        cli_colors: false,
        ..Default::default()
    }
}
