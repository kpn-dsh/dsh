#[macro_use]
extern crate log;

use self::error::DshError;
use clap::Parser;

pub mod config;
mod error;
mod mc;
mod tf;

/// Enum representing the available CLI commands.
///
/// This enum defines the various commands that can be used with the CLI,
/// each variant corresponds to a different subcommand and associated parameters.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
enum Cli {
    /// Command for interacting with the token fetcher.
    ///
    /// The `Tf` variant is used for requesting tokens from the platform.
    /// It takes a `tf::Command` as a parameter, which contains the specific
    /// options and arguments for the token fetcher functionality.
    Tf(tf::Command),

    /// Command for managing configuration values.
    ///
    /// The `Config` variant is used for setting or updating configuration values
    /// for the CLI. It takes a `config::Command` as a parameter, which contains
    /// the specific options and arguments for the configuration functionality.
    Config(config::Command),

    /// Command for creating an MQTT client and connecting to the platform.
    ///
    /// The `Mc` variant is used for managing MQTT client connections to the platform.
    /// It takes a `mc::Command` as a parameter, which contains the specific options
    /// and arguments for the MQTT client functionality.
    Mc(mc::Command),
}

/// The main entry point for the CLI application.
///
/// This asynchronous function initializes the logger, parses the command-line arguments,
/// and dispatches the appropriate functionality based on the provided subcommand.
/// It returns a `Result` to handle any potential errors that may occur during execution.
#[tokio::main]
async fn main() -> Result<(), DshError> {
    // Initialize the logger
    env_logger::init();

    // Parse the command-line arguments into a `Cli` enum
    let args = Cli::parse();

    // Log the parsed arguments for debugging purposes
    debug!("{:?}", &args);

    // Log the current configuration for debugging purposes
    debug!("{:?}", &config::CONFIG.lock().unwrap());

    // Match on the parsed arguments to determine which subcommand to execute,
    // and call the appropriate function with the parsed command parameters.
    match args {
        Cli::Config(cmd) => config::run(&cmd),
        Cli::Tf(cmd) => tf::run(&cmd).await,
        Cli::Mc(cmd) => mc::run(&cmd).await,
    }
}
