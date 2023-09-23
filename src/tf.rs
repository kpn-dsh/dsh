use crate::config;
use crate::error::DshError;
use crate::tf::token::Token;
use clap::Parser;
use futures::{stream, StreamExt};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub mod token;

#[derive(Parser, Debug)]
pub struct Command {
    /// the name of the tenant (will overrule the config)
    #[clap(short, long)]
    pub tenant: Option<String>,
    /// the tenant specific api_key which got the privilege to fetch the tokens (will overrule the config)
    #[clap(short = 'k', long)]
    pub api_key: Option<String>,
    /// the platform api url (for example: poc.kpn-dsh.com) (will overrule the config)
    #[clap(short, long)]
    pub domain: Option<String>,
    /// claims to be added to the token (for example:  '[ { "action": "subscribe", "resource":
    ///   { "stream": "publicstreamname", "prefix": "/tt", "topic": "topicname/#", "type": "topic"
    ///   } } ]')
    #[clap(short, long)]
    pub claims: Option<String>,
    /// amount of tokens to fetch
    #[clap(short = 'a', long, default_value = "1")]
    pub token_amount: usize,
    /// amount of concurrent connections for fetching tokens
    #[clap(short = 'k', long, default_value = "1")]
    pub concurrent_connections: usize,
    /// Location of the output file. If not specified, the output is written to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RequestAttributes {
    pub tenant: String,
    pub api_key: String,
    pub domain: String,
    pub claims: Option<String>,
    pub token_amount: usize,
    pub concurrent_connections: usize,
    pub output: Option<PathBuf>,
}

// return the claims
pub fn get_claims(opt: &Command) -> Result<Option<String>, DshError> {
    match &opt.claims {
        Some(claims) => Ok(Some(claims.to_string())),
        None => Ok(None),
    }
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

// returns the tenant name with the order
// 1 ) the argument given as a parameter
// 2 ) the config
fn get_tenant(opt: &Command) -> Result<String, DshError> {
    match &opt.tenant {
        Some(tenant) => Ok(tenant.to_string()),
        None => {
            let config = config::CONFIG.lock().unwrap();
            if config.tenant.is_empty() {
                Err(DshError::DshCli(
                    "No tenant configured. Please use the config command to set the tenant."
                        .to_string(),
                ))
            } else {
                Ok(config.tenant.to_string())
            }
        }
    }
}

// returns the api_key with the order
// 1 ) the argument given as a parameter
// 2 ) the config
fn get_api_key(opt: &Command) -> Result<String, DshError> {
    match &opt.api_key {
        Some(api_key) => Ok(api_key.to_string()),
        None => {
            let config = config::CONFIG.lock().unwrap();
            if config.api_key.is_empty() {
                Err(DshError::DshCli(
                    "No api_key configured. Please use the config command to set the api_key."
                        .to_string(),
                ))
            } else {
                Ok(config.api_key.to_string())
            }
        }
    }
}

/// This token fetcher (tf) request tokens from the platform
/// this needs either a config or the parameters to be set
async fn request_mqtt_token(
    rest_token: String,
    ra: &RequestAttributes,
) -> Result<Vec<Token>, DshError> {
    let platform = &ra.domain;

    let request_mqtt_token_url = format!("https://api.{platform}/datastreams/v0/mqtt/token",);

    let authorization_header = &*format!("Bearer {}", rest_token);
    debug!("{:?}", &authorization_header);

    let client = reqwest::Client::builder()
        .build()
        .expect("should be able to build reqwest client");

    let urls = vec![&request_mqtt_token_url; ra.token_amount];
    let bodies = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                // claims are applyed in the request of a token
                let map = json!({
                    "id": Uuid::new_v4().to_string(),
                    "tenant": ra.tenant,
                    // if opt claims are set, use them, else don't add claims
                    //
                    // $ dsh tf --claims '[ { "action": "subscribe", "resource": { "stream":
                    //   "ajucpublic", "prefix": "/tt", "topic": "ajuc/test/#", "type": "topic" } }
                    //   ]'
                    //
                    "claims": match &ra.claims {
                        Some(claims) => serde_json::from_str(claims)?,
                        None => serde_json::Value::Null,
                    },
                });
                debug!("json payload request: {:?}", &map);

                let resp = client
                    .post(url)
                    .header("Authorization", &authorization_header.to_string())
                    .json(&map)
                    .send()
                    .await?;

                match resp.status() {
                    reqwest::StatusCode::OK => {
                        let body = resp.text().await?;
                        debug!("response body: {:?}", &body);
                        Ok(body)
                    }
                    _ => Err(DshError::DshCli(format!(
                        "Error requesting token server response code: {:?} body: {:?}",
                        resp.status(),
                        resp.text().await?
                    ))),
                }
            }
        })
        .buffer_unordered(ra.concurrent_connections);

    // mutable vector available in a async blok which contains the tokens
    let tokens = Arc::new(Mutex::new(Vec::new()));

    // create new Token based on body of request and push it to the tokens vector
    bodies
        .for_each(|body| {
            let tokens = Arc::clone(&tokens);
            async move {
                match body {
                    Ok(body) => {
                        let token = Token::new(body);
                        match token {
                            Ok(token) => {
                                let mut tokens = tokens.lock().unwrap();
                                tokens.push(token);
                            }
                            Err(e) => {
                                error!("Error creating token: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error buffered return body: {:?}", e);
                    }
                }
            }
        })
        .await;

    let return_value = tokens.lock().unwrap().to_vec();
    debug!("return_value: {:?}", &return_value);

    Ok(return_value)
}

/// This token fetcher (tf) request tokens from the platform
/// this needs either a config or the parameters to be set
async fn request_rest_token(ra: &RequestAttributes) -> Result<String, DshError> {
    let platform = &ra.domain;
    let tenant = &ra.tenant;
    let api_key = &ra.api_key;

    let request_rest_token_url = format!("https://api.{platform}/auth/v0/token");
    let mut map = std::collections::HashMap::new();
    map.insert("tenant", &tenant);

    let response = reqwest::Client::new()
        .post(&request_rest_token_url)
        .header("apikey", &api_key.to_string())
        .json(&map)
        .send()
        .await?;
    match response.status() {
        reqwest::StatusCode::OK => Ok(response.text().await?),
        _ => {
            let error = response.text().await?;
            Err(error.into())
        }
    }
}

pub async fn run(opt: &Command) -> Result<(), DshError> {
    let request_attributes = RequestAttributes {
        domain: get_platform(opt)?,
        tenant: get_tenant(opt)?,
        api_key: get_api_key(opt)?,
        claims: get_claims(opt)?,
        token_amount: opt.token_amount,
        concurrent_connections: opt.concurrent_connections,
        output: opt.output.clone(),
    };

    let tokens = get_tokens(&request_attributes).await?;
    for token in tokens {
        println!("{}", token.raw_token);
    }
    Ok(())
}

pub async fn get_tokens(request_attributes: &RequestAttributes) -> Result<Vec<Token>, DshError> {
    let rest_token = request_rest_token(request_attributes).await?;
    let tokens = request_mqtt_token(rest_token, request_attributes).await?;
    Ok(tokens)
}
