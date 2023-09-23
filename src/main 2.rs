#[macro_use]
extern crate log;

use self::error::DshError;
use clap::Parser;

pub mod config;
mod error;
mod mc;
mod tf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
enum Cli {
    /// This token fetcher (tf) request tokens from the platform
    Tf(tf::Command),
    /// Set configuration values
    Config(config::Command),
    /// Create mqtt client and connect to the platform
    Mc(mc::Command),
}

#[tokio::main]
async fn main() -> Result<(), DshError> {
    env_logger::init();
    let args = Cli::parse();
    debug!("{:?}", &args);
    // show CONFIG
    debug!("{:?}", &config::CONFIG.lock().unwrap());
    match args {
        Cli::Config(cmd) => config::run(&cmd),
        Cli::Tf(cmd) => tf::run(&cmd).await,
        Cli::Mc(cmd) => mc::run(&cmd).await,
    }
}
