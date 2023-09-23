use crate::error::DshError;
use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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
    /// See the current configuration
    #[clap(short, long)]
    show_all: bool,
}

pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    let c = Config::load(None).unwrap_or_else(|e| {
        eprintln!("Error while loading config: {}", e);
        std::process::exit(1);
    });
    Mutex::new(c)
});

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Config {
    pub tenant: String,
    pub api_key: String,
    pub domain: String,
    pub port: u16,
    pub websocket: bool,
}

impl ::std::default::Default for Config {
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

// implement Config
impl Config {
    // new Config with default values
    pub fn new() -> Self {
        let config = Config {
            ..Default::default()
        };

        debug!("New config with default values: {:?}", config);
        config
    }

    // fn set tenant
    pub fn tenant(&mut self, tenant: &str) -> Result<Config, DshError> {
        self.tenant = tenant.to_string();
        self.save(None)?;
        Ok(self.clone())
    }

    // fn set api_key
    pub fn api_key(&mut self, api_key: &str) -> Result<Config, DshError> {
        self.api_key = api_key.to_string();
        self.save(None)?;
        Ok(self.clone())
    }

    // fn set domain
    pub fn domain(&mut self, domain: &str) -> Result<Config, DshError> {
        self.domain = domain.to_string();
        self.save(None)?;
        Ok(self.clone())
    }

    // fn set port
    pub fn port(&mut self, port: u16) -> Result<Config, DshError> {
        self.port = port;
        self.save(None)?;
        Ok(self.clone())
    }

    // fn set websocket
    pub fn websocket(&mut self, websocket: bool) -> Result<Config, DshError> {
        self.websocket = websocket;
        self.save(None)?;
        Ok(self.clone())
    }

    pub fn load(config_name: Option<&str>) -> Result<Config, DshError> {
        let file = confy::get_configuration_file_path("dsh", config_name)?;
        debug!("Loading config from file: {:?}", file);
        let cfg: Config = confy::load("dsh", config_name)?;
        debug!("Loaded config: {:?}", cfg);
        Ok(cfg)
    }

    pub fn save(&mut self, config_name: Option<&str>) -> Result<(), DshError> {
        debug!("Saving config: {:?}", self);
        confy::store("dsh", config_name, self.clone())?;
        Ok(())
    }
}

pub fn run(opt: &Command) -> Result<(), DshError> {
    // store opt values in config
    let mut config = CONFIG.lock().unwrap();
    if let Some(tenant) = &opt.tenant {
        config.tenant = tenant.to_string();
    }
    if let Some(api_key) = &opt.api_key {
        config.api_key = api_key.to_string();
    }
    if let Some(domain) = &opt.domain {
        config.domain = domain.to_string();
    }
    if let Some(port) = &opt.port {
        config.port = *port;
    }
    if let Some(websocket) = &opt.websocket {
        config.websocket = *websocket;
    }
    if opt.show_all {
        println!("tenant: {}", config.tenant);
        println!("api_key: {}", config.api_key);
        println!("domain: {}", config.domain);
        println!("port: {}", config.port);
        println!("websocket: {}", config.websocket);
    }
    config.save(None)?;
    Ok(())
}

// test config
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert_eq!(config.tenant, "");
        assert_eq!(config.api_key, "");
        assert_eq!(config.domain, "api.poc.kpn-dsh.com".to_string());
        assert_eq!(config.port, 8883);
        assert_eq!(config.websocket, true);
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
        assert_eq!(config.websocket, true);
    }

    #[test]
    fn test_store_config() {
        let mut config = Config::new();
        config.tenant("tenant_name_stored").unwrap();
        config.api_key("api_key_stored").unwrap();
        config.domain("domain_stored").unwrap();
        config.port(111).unwrap();
        config.websocket(true).unwrap();
        config.save(Some("test_store_config")).unwrap();
        let stored_config = Config::load(Some("test_store_config")).unwrap();
        assert_eq!(config, stored_config);
    }

    #[test]
    fn test_store_default_config() {
        let mut config = Config::new();
        config.save(Some("test_store_default_config")).unwrap();
        let stored_config = Config::load(Some("test_store_default_config")).unwrap();
        assert_eq!(config, stored_config);
    }
}
