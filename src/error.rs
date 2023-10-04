/// # DshError Enum
///
/// `DshError` is an enumeration of all possible errors that can occur
/// while executing a DSH CLI command or while working with the `dsh` crate.
/// It is designed to wrap various error types into a single unified type
/// to be used throughout the application for better error handling and management.
///
/// ## Variants
///
/// - `SerdeJson`: Errors related to serialization and deserialization using `serde_json`.
/// - `Base64`: Errors related to Base64 encoding and decoding.
/// - `Request`: Errors that may occur during HTTP requests using `reqwest`.
/// - `DshCli`: Custom errors specific to DSH CLI, represented as a string.
/// - `PortNotPresentInToken`: Error when a specified port is not present in a token.
/// - `SecureStore`: Errors related to secure storage operations.
/// - `Io`: Standard input/output errors.
/// - `Client`: Errors related to MQTT client operations using `rumqttc`.
/// - `Mqtt`: General MQTT errors using `rumqttc`.
/// - `MqttConnection`: Errors related to MQTT connection using `rumqttc`.
/// - `Confy`: Errors related to configuration management using `confy`.
/// - `KeyringError`: Errors related to keyring operations.
///
/// ## Implementations
///
/// `DshError` implements various `From` traits to allow for easy conversion
/// from other error types to `DshError`, providing a seamless way to propagate
/// errors up the call stack and convert them into a `DshError` variant.
///
/// ## Display
///
/// It also implements the `Display` trait to facilitate user-friendly error messages
/// when displaying or logging errors.
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

/// # Display Implementation for DshError
///
/// This implementation of the `std::fmt::Display` trait allows for
/// user-friendly printing of `DshError` variants. Each variant is
/// matched and a formatted string is returned, providing a clear
/// and descriptive error message.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_serde_json_error() {
        let serde_result: Result<serde_json::Value, _> = serde_json::from_str("Not a valid JSON");
        let serde_error = serde_result.unwrap_err();
        let dsh_error: DshError = serde_error.into();
        match dsh_error {
            DshError::SerdeJson(_) => (),
            _ => panic!("Incorrect error variant"),
        }
    }

    #[test]
    fn test_from_base64_error() {
        let base64_error = base64::DecodeError::InvalidLength;
        let dsh_error: DshError = base64_error.into();
        match dsh_error {
            DshError::Base64(_) => (),
            _ => panic!("Incorrect error variant"),
        }
    }

    // TODO add more test

    #[test]
    fn test_display() {
        let serde_result: Result<serde_json::Value, _> = serde_json::from_str("Not a valid JSON");
        let serde_error = serde_result.unwrap_err();
        let dsh_error: DshError = serde_error.into();
        assert_eq!(
            format!("{}", dsh_error),
            "SerdeJsonError: expected value at line 1 column 1"
        );

        let base64_error = base64::DecodeError::InvalidLength;
        let dsh_error: DshError = base64_error.into();
        assert_eq!(
            format!("{}", dsh_error),
            "Base64 error: Encoded text cannot have a 6-bit remainder."
        );

        // ... Similar tests for other error types ...
    }
}
