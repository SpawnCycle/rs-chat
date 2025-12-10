use rocket::Config;

pub fn rocket() -> Config {
    Config {
        cli_colors: false,
        ..Default::default()
    }
}
