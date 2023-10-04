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

/// Represents command-line arguments and options for the Command.
///
/// This struct is derived from clap's Parser and contains various options
/// that can be specified by the user in the command line interface.
#[derive(Parser, Debug, Default)]
pub struct Command {
    /// The name of the tenant.
    ///
    /// This will override the tenant name specified in the configuration.
    #[clap(short, long)]
    pub tenant: Option<String>,

    /// The tenant-specific API key with privileges to fetch the tokens.
    ///
    /// This will override the API key specified in the configuration.
    #[clap(short = 'k', long)]
    pub api_key: Option<String>,

    /// The platform API URL (e.g., poc.kpn-dsh.com).
    ///
    /// This will override the domain specified in the configuration.
    #[clap(short, long)]
    pub domain: Option<String>,

    /// Claims to be added to the token.
    ///
    /// Example: '[ { "action": "subscribe", "resource": { "stream": "publicstreamname", "prefix": "/tt", "topic": "topicname/#", "type": "topic" } } ]'
    #[clap(short, long)]
    pub claims: Option<String>,

    /// The number of tokens to fetch.
    #[clap(short = 'a', long, default_value = "1")]
    pub token_amount: usize,

    /// The number of concurrent connections for fetching tokens.
    #[clap(short = 'k', long, default_value = "1")]
    pub concurrent_connections: usize,

    /// The location of the output file.
    ///
    /// If not specified, the output is written to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

/// Contains attributes required for making requests.
///
/// This struct is used to pass around request-related attributes
/// and options in a type-safe manner.
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

/// Retrieve the claims specified in the Command options.
///
/// # Arguments
///
/// * `opt` - A reference to the Command struct containing possible user-specified claims.
///
/// # Returns
///
/// * `Result<Option<String>, DshError>` - The claims as a JSON string if specified, otherwise None.
pub fn get_claims(opt: &Command) -> Result<Option<String>, DshError> {
    match &opt.claims {
        Some(claims) => Ok(Some(claims.to_string())),
        None => Ok(None),
    }
}

/// Get the platform domain based on user input or configuration.
///
/// This function retrieves the platform domain URL according to the following order of precedence:
/// 1. Utilizes the platform domain provided as an argument to the function (if provided).
/// 2. If no argument is provided, it retrieves the platform domain from the configuration.
///
/// # Arguments
///
/// * `opt` - A reference to the Command struct containing possible user-specified options and arguments.
///
/// # Returns
///
/// * `Result<String, DshError>` - The platform domain as a string if found, otherwise returns an error.
///
/// # Examples
///
/// ```
/// // Example usage of `get_platform`:
/// let platform_domain = get_platform(&command_options)?;
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - Neither the argument nor the configuration provides a valid platform domain.
/// - There are issues accessing or reading the configuration.
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

/// Retrieve the tenant name based on user input or configuration.
///
/// This function determines the tenant name using the following priority:
/// 1. Uses the tenant name provided as an argument to the function (if provided).
/// 2. If no argument is provided, it retrieves the tenant name from the configuration.
///
/// # Arguments
///
/// * `opt` - A reference to the Command struct containing possible user-specified options and arguments.
///
/// # Returns
///
/// * `Result<String, DshError>` - The tenant name as a string if found, otherwise returns an error.
///
/// # Examples
///
/// ```
/// // Example usage of `get_tenant`:
/// let tenant_name = get_tenant(&command_options)?;
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - Neither the argument nor the configuration provides a valid tenant name.
/// - There are issues accessing or reading the configuration.
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

/// Retrieve the user's API key for platform access.
///
/// This function obtains the user's API key by checking:
/// 1. The API key provided as a function argument (if any).
/// 2. The API key stored in the configuration if no argument is provided.
///
/// # Arguments
///
/// * `opt` - A reference to the Command struct containing possible user-specified options and arguments.
///
/// # Returns
///
/// * `Result<String, DshError>` - The API key as a string if found, otherwise returns an error.
///
/// # Errors
///
/// This function will return an error if:
/// - Neither the argument nor the configuration provides a valid API key.
/// - There are issues accessing or reading the configuration.
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

/// Request MQTT tokens from the platform.
///
/// This asynchronous function sends a request to the platform to retrieve MQTT tokens.
/// It requires either a configuration or parameters to be set.
///
/// # Arguments
///
/// * `rest_token` - A String containing the REST token used for authorization.
/// * `ra` - A reference to the RequestAttributes struct containing request parameters like domain, tenant, etc.
///
/// # Returns
///
/// * `Result<Vec<Token>, DshError>` - A vector of Token structs if the request is successful, otherwise returns an error.
///
/// # Examples
///
/// ```
/// // Example usage of `request_mqtt_token`:
/// let mqtt_tokens = request_mqtt_token(rest_token_string, &request_attributes).await?;
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The platform returns a non-OK status code.
/// - There are issues with sending the request or parsing the response.
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

/// Request a REST token from the platform.
///
/// This asynchronous function sends a request to the platform to retrieve a REST token.
/// It requires either a configuration or parameters to be set.
///
/// # Arguments
///
/// * `ra` - A reference to the RequestAttributes struct containing request parameters like domain, tenant, etc.
///
/// # Returns
///
/// * `Result<String, DshError>` - A string containing the REST token if the request is successful, otherwise returns an error.
///
/// # Examples
///
/// ```
/// // Example usage of `request_rest_token`:
/// let rest_token = request_rest_token(&request_attributes).await?;
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The platform returns a non-OK status code.
/// - There are issues with sending the request or parsing the response.
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

/// Main function to run the token fetcher.
///
/// # Arguments
///
/// * `opt` - A reference to the Command struct containing user-specified options and arguments.
///
/// # Returns
///
/// * `Result<(), DshError>` - Returns Ok(()) if successful, otherwise returns an error.
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

/// Fetches tokens based on the specified request attributes.
///
/// # Arguments
///
/// * `request_attributes` - A reference to the RequestAttributes struct containing request-related attributes and options.
///
/// # Returns
///
/// * `Result<Vec<Token>, DshError>` - A vector of fetched tokens if successful, otherwise returns an error.
pub async fn get_tokens(request_attributes: &RequestAttributes) -> Result<Vec<Token>, DshError> {
    let rest_token = request_rest_token(request_attributes).await?;
    let tokens = request_mqtt_token(rest_token, request_attributes).await?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_claims_with_some() {
        let cmd = Command {
            claims: Some(String::from("test_claims")),
            ..Default::default() // assuming you derive Default for Command
        };
        assert_eq!(get_claims(&cmd).unwrap(), Some(String::from("test_claims")));
    }

    #[test]
    fn test_get_claims_with_none() {
        let cmd = Command {
            claims: None,
            ..Default::default()
        };
        assert_eq!(get_claims(&cmd).unwrap(), None);
    }

    #[test]
    fn test_get_platform_with_domain() {
        let cmd = Command {
            domain: Some(String::from("test_domain")),
            ..Default::default()
        };
        assert_eq!(get_platform(&cmd).unwrap(), String::from("test_domain"));
    }

    #[test]
    fn test_get_platform_without_domain() {
        let cmd = Command {
            domain: None,
            ..Default::default()
        };
        // Assuming you have a domain in your config
        assert_eq!(
            get_platform(&cmd).unwrap(),
            String::from("api.poc.kpn-dsh.com")
        );
    }

    #[test]
    fn test_get_tenant_with_tenant() {
        let cmd = Command {
            tenant: Some(String::from("test_tenant")),
            ..Default::default()
        };
        assert_eq!(get_tenant(&cmd).unwrap(), String::from("test_tenant"));
    }

    #[test]
    fn test_get_tenant_without_tenant() {
        // Act
        let cmd = Command {
            tenant: None,
            ..Default::default()
        };

        let result = get_tenant(&cmd);

        // Assert
        assert!(
            result.is_err(),
            "Expected an error due to missing tenant configuration."
        );

        let err_msg = result.unwrap_err().to_string();
        let expected_err_msg =
            "DshCli error: No tenant configured. Please use the config command to set the tenant.";
        assert_eq!(err_msg, expected_err_msg, "Unexpected error message.");
    }

    #[test]
    fn test_get_api_key_with_key() {
        let cmd = Command {
            api_key: Some(String::from("test_key")),
            ..Default::default()
        };
        assert_eq!(get_api_key(&cmd).unwrap(), String::from("test_key"));
    }

    #[test]
    fn test_get_api_key_without_key() {
        // Act
        let cmd = Command {
            api_key: None,
            ..Default::default()
        };

        let result = get_api_key(&cmd);

        // Assert
        assert!(result.is_err(), "Expected an error due to missing API key.");

        let err_msg = result.unwrap_err().to_string();
        let expected_err_msg = "DshCli error: No api_key configured. Please use the config command to set the api_key.";
        assert_eq!(err_msg, expected_err_msg, "Unexpected error message.");
    }
}
