use crate::error::DshError;
use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Mutex;
use std::sync::RwLock;

// Define static constants and configurations
static CACHED_CONFIG: Lazy<RwLock<Option<Config>>> = Lazy::new(|| RwLock::new(None));
const SERVICE_NAME: &str = "dsh";
const CONFIG_KEY: &str = "dsh_config";

/// Represents the command-line arguments and options for the application.
#[derive(Parser, Debug)]
pub struct Command {
    /// Set the name of the tenant
    #[clap(short, long)]
    tenant: Option<String>,
    /// Set the tenant specific api_key which got the privilege to fetch the tokens
    #[clap(short = 'k', long)]
    api_key: Option<String>,
    /// Set the platform api url (for example: poc.kpn-dsh.com)
    #[clap(short, long)]
    domain: Option<String>,
    /// Set the platform mqtt client port (for example: 8883)
    #[clap(short, long)]
    port: Option<u16>,
    /// Set if connection goes over websocket
    #[clap(short, long)]
    websocket: Option<bool>,
    /// See the current configuration, including the full unmasked API-key
    #[clap(short, long)]
    show_all: bool,
    /// Clean the OS secret store
    #[clap(short, long)]
    clean_secret_store: bool,
}

// Global configuration instance
pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    let c = Config::load(None).unwrap_or_else(|e| {
        eprintln!("Error while loading config: {}", e);
        std::process::exit(1);
    });
    Mutex::new(c)
});

/// A configuration structure used for managing settings.
///
/// This structure holds various configuration parameters used in the application, such as API keys, domain names, etc.
///
/// # Examples
///
/// ```
/// let config = Config {
///     tenant: String::from("example_tenant"),
///     api_key: String::from("secret_api_key"),
///     domain: String::from("example.com"),
///     port: 8080,
///     websocket: false,
/// };
/// println!("{}", config);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Config {
    pub tenant: String,
    pub api_key: String,
    pub domain: String,
    pub port: u16,
    pub websocket: bool,
}

// Default values for Config
impl std::default::Default for Config {
    fn default() -> Self {
        Config {
            tenant: "".to_string(),
            api_key: "".to_string(),
            domain: "api.poc.kpn-dsh.com".to_string(),
            port: 8883,
            websocket: true,
        }
    }
}

// Implementation of Config methods
impl Config {
    /// Create a new Config with default values
    pub fn new() -> Self {
        let config = Config {
            ..Default::default()
        };

        debug!("New config with default values: {:?}", config);
        config
    }

    // Setter methods for Config fields
    pub fn tenant(&mut self, tenant: &str) -> Result<Config, DshError> {
        self.tenant = tenant.to_string();
        self.save(None)?;
        Ok(self.clone())
    }

    pub fn api_key(&mut self, api_key: &str) -> Result<Config, DshError> {
        self.api_key = api_key.to_string();
        self.save(None)?;
        Ok(self.clone())
    }

    pub fn domain(&mut self, domain: &str) -> Result<Config, DshError> {
        self.domain = domain.to_string();
        self.save(None)?;
        Ok(self.clone())
    }

    pub fn port(&mut self, port: u16) -> Result<Config, DshError> {
        self.port = port;
        self.save(None)?;
        Ok(self.clone())
    }

    pub fn websocket(&mut self, websocket: bool) -> Result<Config, DshError> {
        self.websocket = websocket;
        self.save(None)?;
        Ok(self.clone())
    }

    /// Save the current configuration to the OS secret store
    pub fn save(&mut self, config_name: Option<&str>) -> Result<(), DshError> {
        let serialized_config = serde_json::to_string(&self)?;

        // Use the provided config_name or fall back to the default CONFIG_KEY
        let key_name = config_name.unwrap_or(CONFIG_KEY);

        let entry = keyring::Entry::new(SERVICE_NAME, key_name)?;
        entry.set_password(&serialized_config)?;

        Ok(())
    }

    /// Clean the OS secret store for the given config_name
    pub fn clean_secret_store(config_name: Option<&str>) -> Result<(), DshError> {
        let key_name = config_name.unwrap_or(CONFIG_KEY);
        let entry = keyring::Entry::new(SERVICE_NAME, key_name)?;

        let mut cache = CACHED_CONFIG.write().expect("Failed to obtain write lock");
        *cache = None;

        match entry.get_password() {
            Ok(_) => {
                // If password retrieval is successful, delete the entry
                entry.delete_password()?;
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                // If there's no entry, do nothing and return Ok
                Ok(())
            }
            Err(e) => Err(DshError::from(e)), // Handle other errors
        }
    }

    /// Load the configuration from the OS secret store or cache
    pub fn load(config_name: Option<&str>) -> Result<Config, DshError> {
        // Check if the configuration is already cached
        {
            let cached_config_read = CACHED_CONFIG.read().unwrap();
            if let Some(cached_config) = &*cached_config_read {
                return Ok(cached_config.clone());
            }
        }

        // If not cached, fetch from the OS secret store
        let key_name = config_name.unwrap_or(CONFIG_KEY);
        let entry = keyring::Entry::new(SERVICE_NAME, key_name)?;
        let serialized_config = match entry.get_password() {
            Ok(config) => config,
            Err(keyring::Error::NoEntry) => {
                let mut new_entry = Config::default();
                new_entry.save(Some(key_name))?;
                return Ok(new_entry);
            }
            Err(e) => return Err(DshError::from(e)),
        };
        let config: Config = serde_json::from_str(&serialized_config)?;

        // Cache the fetched configuration
        {
            let mut cached_config_write = CACHED_CONFIG.write().unwrap();
            *cached_config_write = Some(config.clone());
        }

        Ok(config)
    }
}

/// Implementing display trait for Config struct.
///
/// This implementation allows for pretty-printing of `Config` instances,
/// while also ensuring that sensitive information (like the API key) is masked when printed.
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mask the API key, showing only the last 4 characters.
        //
        // If the API key is shorter than 4 characters, it will be fully masked.
        // Otherwise, all but the last 4 characters will be replaced with asterisks (`*`).
        let masked_api_key = if self.api_key.len() > 4 {
            format!(
                "{}{}",
                "*".repeat(self.api_key.len() - 4),
                &self.api_key[self.api_key.len() - 4..]
            )
        } else {
            "*".repeat(self.api_key.len())
        };

        // Write the formatted `Config` instance to the provided formatter.
        //
        // The `Config` instance will be written in the following format:
        //
        // ```plaintext
        // Tenant: [tenant]
        // API Key: [masked_api_key]
        // Domain: [domain]
        // Port: [port]
        // Websocket: [websocket]
        // ```
        write!(
            f,
            "Tenant: {}\nAPI Key: {}\nDomain: {}\nPort: {}\nWebsocket: {}",
            self.tenant, masked_api_key, self.domain, self.port, self.websocket
        )
    }
}

// Main function to run the application based on the provided command-line options
pub fn run(opt: &Command) -> Result<(), DshError> {
    // store opt values in config
    let mut config = CONFIG.lock().unwrap();
    let mut any_option_set = false; // Flag to check if any option is set

    if let Some(tenant) = &opt.tenant {
        config.tenant = tenant.to_string();
        any_option_set = true;
    }
    if let Some(api_key) = &opt.api_key {
        config.api_key = api_key.to_string();
        any_option_set = true;
    }
    if let Some(domain) = &opt.domain {
        config.domain = domain.to_string();
        any_option_set = true;
    }
    if let Some(port) = &opt.port {
        config.port = *port;
        any_option_set = true;
    }
    if let Some(websocket) = &opt.websocket {
        config.websocket = *websocket;
        any_option_set = true;
    }
    if opt.show_all {
        println!(
            "Tenant: {}\nAPI Key: {}\nDomain: {}\nPort: {}\nWebsocket: {}",
            config.tenant, config.api_key, config.domain, config.port, config.websocket
        );
        any_option_set = true;
    }
    if opt.clean_secret_store {
        return Config::clean_secret_store(None);
    }
    if !any_option_set {
        println!("{}", config);
    }
    config.save(None)?;
    Ok(())
}

// Unit tests for the Config struct
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "mock_os_secret_store")]
    use keyring::{mock, set_default_credential_builder};

    const TEST_CONFIG_NAME: &str = "test_dsh_config";

    fn setup() {
        #[cfg(feature = "mock_os_secret_store")]
        set_default_credential_builder(mock::default_credential_builder());

        // Clean the secret store with the test-specific config_name before each test
        Config::clean_secret_store(Some(TEST_CONFIG_NAME)).unwrap();
    }

    fn teardown() {
        // Clean the secret store with the test-specific config_name after each test
        Config::clean_secret_store(Some(TEST_CONFIG_NAME)).unwrap();
    }

    #[test]
    fn test_default_config() {
        setup();
        let config = Config::new();
        assert_eq!(config.tenant, "");
        assert_eq!(config.api_key, "");
        assert_eq!(config.domain, "api.poc.kpn-dsh.com".to_string());
        assert_eq!(config.port, 8883);
        assert!(config.websocket);
        teardown();
    }

    #[test]
    fn test_set_tenant() {
        let mut config = Config::new();
        config.tenant("tenant_name").unwrap();
        assert_eq!(config.tenant, "tenant_name".to_string());
    }

    #[test]
    fn test_set_domain() {
        let mut config = Config::new();
        config.domain("domain").unwrap();
        assert_eq!(config.domain, "domain".to_string());
    }

    #[test]
    fn test_set_port() {
        let mut config = Config::new();
        config.port(1234).unwrap();
        assert_eq!(config.port, 1234);
    }

    #[test]
    fn test_set_api_key() {
        let mut config = Config::new();
        config.api_key("api_key").unwrap();
        assert_eq!(config.api_key, "api_key".to_string());
    }

    #[test]
    fn test_set_websocket() {
        let mut config = Config::new();
        config.websocket(true).unwrap();
        assert!(config.websocket);
    }

    #[test]
    fn test_load_empty_secret_store() {
        setup();
        let stored_config = Config::load(Some(TEST_CONFIG_NAME)).unwrap();
        let config = Config::new();
        assert_eq!(config, stored_config);
        teardown();
    }

    #[test]
    fn test_store_config() {
        setup();
        let mut config = Config::new();
        config.tenant("tenant_name_stored").unwrap();
        config.api_key("api_key_stored").unwrap();
        config.domain("domain_stored").unwrap();
        config.port(111).unwrap();
        config.websocket(true).unwrap();
        config.save(Some(TEST_CONFIG_NAME)).unwrap();
        // TODO: it would be great to add a statefull version of mocking this test and validate the
        // stored config from the mock secret store
        teardown();
    }

    #[test]
    fn test_store_default_config() {
        setup();
        let mut config = Config::new();
        config.save(Some(TEST_CONFIG_NAME)).unwrap();
        let stored_config = Config::load(Some(TEST_CONFIG_NAME)).unwrap();
        assert_eq!(config, stored_config);
        teardown();
    }
}
