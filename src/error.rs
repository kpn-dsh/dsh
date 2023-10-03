/// The Errors that may occur when processing a DSH Cli command
/// It is used to wrap all errors that can occur in the dsh crate.
#[derive(Debug)]
pub enum DshError {
    SerdeJson(serde_json::Error),
    Base64(base64::DecodeError),
    Request(reqwest::Error),
    DshCli(String),
    PortNotPresentInToken(u16),
    SecureStore(securestore::Error),
    Io(std::io::Error),
    Client(rumqttc::ClientError),
    Mqtt(rumqttc::Error),
    MqttConnection(rumqttc::ConnectionError),
    Confy(confy::ConfyError),
    KeyringError(keyring::Error),
}

/// From ConfyError
impl From<confy::ConfyError> for DshError {
    fn from(e: confy::ConfyError) -> Self {
        DshError::Confy(e)
    }
}

/// From std::io::Error
impl From<std::io::Error> for DshError {
    fn from(err: std::io::Error) -> DshError {
        DshError::Io(err)
    }
}

/// From SerdeJsonError
impl From<serde_json::Error> for DshError {
    fn from(error: serde_json::Error) -> Self {
        DshError::SerdeJson(error)
    }
}

/// From Base64Error
impl From<base64::DecodeError> for DshError {
    fn from(e: base64::DecodeError) -> DshError {
        DshError::Base64(e)
    }
}

/// From ReqwestError
impl From<reqwest::Error> for DshError {
    fn from(e: reqwest::Error) -> Self {
        DshError::Request(e)
    }
}

/// From SecureStoreError
impl From<securestore::Error> for DshError {
    fn from(e: securestore::Error) -> Self {
        DshError::SecureStore(e)
    }
}

/// From &str
impl From<&str> for DshError {
    fn from(e: &str) -> Self {
        DshError::DshCli(e.to_string())
    }
}

/// From String
impl From<String> for DshError {
    fn from(e: String) -> Self {
        DshError::DshCli(e)
    }
}

/// From ClientError
impl From<rumqttc::ClientError> for DshError {
    fn from(e: rumqttc::ClientError) -> Self {
        DshError::Client(e)
    }
}

/// From MqttError
impl From<rumqttc::Error> for DshError {
    fn from(e: rumqttc::Error) -> Self {
        DshError::Mqtt(e)
    }
}

/// From MqttConnectionError
impl From<rumqttc::ConnectionError> for DshError {
    fn from(e: rumqttc::ConnectionError) -> Self {
        DshError::MqttConnection(e)
    }
}

/// From PortNotPresentInToken
impl From<u16> for DshError {
    fn from(e: u16) -> Self {
        DshError::PortNotPresentInToken(e)
    }
}

/// From KeyringError
impl From<keyring::Error> for DshError {
    fn from(error: keyring::Error) -> Self {
        DshError::KeyringError(error)
    }
}

impl std::fmt::Display for DshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DshError::SerdeJson(e) => write!(f, "SerdeJsonError: {}", e),
            DshError::Base64(e) => write!(f, "Base64 error: {}", e),
            DshError::Request(e) => write!(f, "Reqwest error: {}", e),
            DshError::DshCli(e) => write!(f, "DshCli error: {}", e),
            DshError::SecureStore(e) => write!(f, "SecureStore error: {}", e),
            DshError::Io(e) => write!(f, "Io error: {}", e),
            DshError::Client(e) => write!(f, "Client error: {}", e),
            DshError::Mqtt(e) => write!(f, "Mqtt error: {}", e),
            DshError::MqttConnection(e) => write!(f, "Mqtt connection error: {}", e),
            DshError::Confy(e) => write!(f, "Confy error: {}", e),
            DshError::PortNotPresentInToken(e) => write!(f, "Port not present in token: {}", e),
            DshError::KeyringError(e) => write!(f, "Keyring Error: {}", e),
        }
    }
}
