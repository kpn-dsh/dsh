use crate::config;
use crate::error::DshError;
use crate::tf::token::Token;
use clap::Parser;
use std::path::PathBuf;

mod client;

#[derive(Parser, Debug)]
pub struct Command {
    /// MQTT topic (for example: "/tt/topicname/")
    #[clap(short, long)]
    topic: String,
    /// MQTT client id (will override value from token)
    #[clap(long)]
    client_id: Option<String>,
    /// MQTT broker address (will override value from token)
    #[clap(short, long)]
    domain: Option<String>,
    /// MQTT broker port (will override value from token)
    #[clap(short, long)]
    port: Option<u16>,
    /// api key
    #[clap(short, long)]
    api_key: Option<String>,
    /// tenant name
    #[clap(short, long)]
    tenant: Option<String>,
    /// claims to be added to the token (for example:  '[ { "action": "subscribe", "resource":
    /// { "stream": "publicstreamname", "prefix": "/tt", "topic": "topicname/#", "type": "topic" }
    /// } ]'
    #[clap(long)]
    claims: Option<String>,
    /// MQTT message. If provided only the message will be sent and the app will exit (read no
    /// consumption)
    #[clap(short, long)]
    message: Option<String>,
    /// Connection over websockets
    // default is enforced via function not via clap
    #[clap(short, long)]
    websocket: bool,
    /// Verbose heatbeat messages
    #[clap(short, long)]
    verbose_heartbeat: bool,
    /// Concise output (only print topic and message)
    #[clap(short, long)]
    concise: bool,
}

// run
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

// return the claims with the order
// 1 ) the argument given as a parameter
pub fn get_claims(opt: &Command) -> Result<Option<String>, DshError> {
    match &opt.claims {
        Some(claims) => Ok(Some(claims.to_string())),
        None => Ok(None),
    }
}

// return token amount of 1
pub fn get_token_amount() -> Result<usize, DshError> {
    Ok(1)
}

// return concurrent connections of 1
pub fn get_concurrent_connections() -> Result<usize, DshError> {
    Ok(1)
}

// return output of None
pub fn get_output() -> Result<Option<PathBuf>, DshError> {
    Ok(None)
}

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
fn get_topic(opt: &Command) -> Result<String, DshError> {
    let topic = opt.topic.clone();

    // add /tt prefix to topic
    if topic.starts_with('/') {
        Ok(format!("/tt{}", topic))
    } else {
        Ok(format!("/tt/{}", topic))
    }
}
