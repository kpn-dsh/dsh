use crate::config;
use crate::error::DshError;
use crate::tf::token::Token;
use clap::Parser;
use std::path::PathBuf;

mod client;

/// Represents the command-line arguments and options for the application.
#[derive(Parser, Debug)]
pub struct Command {
    /// Specifies the MQTT topic, e.g., "/tt/topicname/".
    #[clap(short, long)]
    topic: String,
    /// Optionally overrides the MQTT client ID from the token.
    #[clap(long)]
    client_id: Option<String>,
    /// Optionally overrides the MQTT broker address from the token.
    #[clap(short, long)]
    domain: Option<String>,
    /// Optionally overrides the MQTT broker port from the token.
    #[clap(short, long)]
    port: Option<u16>,
    /// Optionally overrides the API key for authentication.
    #[clap(short, long)]
    api_key: Option<String>,
    /// tenant name
    #[clap(short, long)]
    tenant: Option<String>,
    /// Claims to be added to the token, e.g., for specifying permissions.
    /// for example:  '[ { "action": "subscribe", "resource": { "stream": "publicstreamname",
    /// "prefix": "/tt", "topic": "topicname/#", "type": "topic" } } ]'
    #[clap(long)]
    claims: Option<String>,
    /// MQTT message to be sent. If provided, only this message will be sent and the app will exit.
    #[clap(short, long)]
    message: Option<String>,
    /// Specifies whether to connect via websockets. Default is determined by a function, not clap.
    #[clap(short, long)]
    websocket: bool,
    /// Enables verbose heartbeat messages if set.
    #[clap(short, long)]
    verbose_heartbeat: bool,
    /// Enables concise output, printing only topic and message, if set.
    #[clap(short, long)]
    concise: bool,
}

/// Executes the main logic based on the provided command-line options.
pub async fn run(opt: &Command) -> Result<(), DshError> {
    debug!("Commands input: {:?}", opt);

    // get attributes
    let token = get_token(opt).await?;
    let port = get_port(opt)?;
    let topic = get_topic(opt)?;
    let websocket = get_websocket(opt)?;
    let concise = opt.concise;
    let verbose = opt.verbose_heartbeat;
    let message = opt.message.clone();

    let client =
        client::Client::new(token, port, topic, websocket, verbose, concise, message).await?;
    client.connect().await?;

    Ok(())
}

// returns the platform domain url with the order
// 1 ) the argument given as a parameter
// 2 ) the config
/// Determines the platform domain URL, prioritizing the command-line argument, then the config.
fn get_platform(opt: &Command) -> Result<String, DshError> {
    match &opt.domain {
        Some(domain) => Ok(domain.to_string()),
        None => {
            let config = config::CONFIG.lock().unwrap();
            if config.domain.is_empty() {
                Err(DshError::DshCli(
                    "No domain configured. Please use the config command to set the domain."
                        .to_string(),
                ))
            } else {
                Ok(config.domain.to_string())
            }
        }
    }
}

// return the tenant with the order
// 1 ) the argument given as a parameter
// 1 ) the config
/// Determines the tenant, prioritizing the command-line argument, then the config.
fn get_tenant(opt: &Command) -> Result<String, DshError> {
    match &opt.tenant {
        Some(tenant) => Ok(tenant.to_string()),
        None => {
            let config = config::CONFIG.lock().unwrap();
            if config.domain.is_empty() {
                Err(DshError::DshCli(
                    "No tenant configuration. Please us the config command to set the tenant."
                        .to_string(),
                ))
            } else {
                Ok(config.tenant.to_string())
            }
        }
    }
}

// return the api key with the order
// 1 ) the argument given as a parameter
// 1 ) the config
/// Determines the API key, prioritizing the command-line argument, then the config.
fn get_api_key(opt: &Command) -> Result<String, DshError> {
    match &opt.api_key {
        Some(api_key) => Ok(api_key.to_string()),
        None => {
            let config = config::CONFIG.lock().unwrap();
            if config.domain.is_empty() {
                Err(DshError::DshCli(
                    "No api key configured. Please use the config command to set the api key."
                        .to_string(),
                ))
            } else {
                Ok(config.api_key.to_string())
            }
        }
    }
}

// return if websocket should be used
// 1 ) the argument given as a parameter
// 1 ) the config
/// Determines whether to use websockets, prioritizing the command-line argument, then the config.
fn get_websocket(opt: &Command) -> Result<bool, DshError> {
    match &opt.websocket {
        true => Ok(true),
        false => {
            let config = config::CONFIG.lock().unwrap();
            if config.domain.is_empty() {
                Err(DshError::DshCli(
                    "No websockets configuration. Please us the config command to set the websockets."
                        .to_string(),
                ))
            } else {
                Ok(config.websocket)
            }
        }
    }
}

/// Retrieves the claims from the command-line argument.
pub fn get_claims(opt: &Command) -> Result<Option<String>, DshError> {
    match &opt.claims {
        Some(claims) => Ok(Some(claims.to_string())),
        None => Ok(None),
    }
}

/// Return token amount of 1 because this is a single client
pub fn get_token_amount() -> Result<usize, DshError> {
    Ok(1)
}

/// Returns concurrent connections of 1, because this is a single client
pub fn get_concurrent_connections() -> Result<usize, DshError> {
    Ok(1)
}

/// Returns an output of None, because we want to be a MQTT client
pub fn get_output() -> Result<Option<PathBuf>, DshError> {
    Ok(None)
}

/// Retrieves a token, prioritizing the command-line argument, then the config.
pub async fn get_token(opt: &Command) -> Result<Token, DshError> {
    let ra = super::tf::RequestAttributes {
        domain: get_platform(opt)?,
        tenant: get_tenant(opt)?,
        api_key: get_api_key(opt)?,
        token_amount: get_token_amount()?,
        concurrent_connections: get_concurrent_connections()?,
        output: get_output()?,
        claims: get_claims(opt)?,
    };
    debug!("Request attributes: {:#?}", ra);

    let tokens: Vec<Token> = super::tf::get_tokens(&ra).await?;

    if tokens.is_empty() {
        Err(DshError::DshCli("No token received".to_string()))
    } else {
        Ok(tokens[0].clone())
    }
}

// returns the platform port with the order
// 1 ) the argument given as a parameter
// 2 ) the config
// TODO: validate if port is in to be provided token
/// Determines the platform port, prioritizing the command-line argument, then the config.
fn get_port(opt: &Command) -> Result<u16, DshError> {
    match &opt.port {
        Some(port) => Ok(*port),
        None => {
            let config = config::CONFIG.lock().unwrap();
            if config.port == 0 {
                Err(DshError::DshCli(
                    "No port configured. Please use the config command to set the port."
                        .to_string(),
                ))
            } else {
                Ok(config.port)
            }
        }
    }
}

// returns the propaly formated topic
/// Formats the topic properly, ensuring it starts with "/tt".
fn get_topic(opt: &Command) -> Result<String, DshError> {
    let topic = opt.topic.clone();

    // add /tt prefix to topic
    if topic.starts_with('/') {
        Ok(format!("/tt{}", topic))
    } else {
        Ok(format!("/tt/{}", topic))
    }
}
