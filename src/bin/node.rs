use clap::Parser;
use std::env;
use su_chain::blockchain::{App, Blockchain};
use su_chain::config::loader::load_toml;
use su_chain::config::models::Config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value_t = String::from("./config/config.toml")
    )]
    config_path: String,

    #[arg(short, long, default_value_t = String::from("debug"))]
    log_level: String,
}

fn load_config() -> Config {
    let cli = Args::parse();
    let config = load_toml(cli.config_path.as_str());

    env::set_var("RUST_LOG", cli.log_level);
    config
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();

    let config = load_config();
    env_logger::init();

    let app_name = env!("CARGO_PKG_NAME");

    log::info!("Application '{}' started.", app_name);
    log::debug!("{:#?}", config);

    let mut app = App::new();
    app.run();

    Ok(())
}
