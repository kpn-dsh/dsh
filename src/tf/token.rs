use crate::error::DshError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Token {
    pub raw_token: String,
    pub token_attributes: TokenAttributes,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TokenAttributes {
    gen: i32,
    pub endpoint: String,
    iss: String,
    pub claims: Vec<Claims>,
    exp: i32,
    pub ports: Ports,
    pub client_id: String,
    iat: i32,
    pub tenant_id: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Claims {
    resource: Resource,
    action: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Resource {
    stream: String,
    prefix: String,
    topic: String,
    type_: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Ports {
    pub mqtts: Vec<u16>,
    pub mqttwss: Vec<u16>,
}

// create new Token based on a given String
impl Token {
    pub fn new(raw_token: String) -> Result<Token, DshError> {
        use base64::{alphabet, engine, read};
        use std::io::Read;

        // Split token and get the [1] part
        let split_token = raw_token.split('.').collect::<Vec<&str>>();

        // Create an instance of the GeneralPurpose engine with the STANDARD alphabet
        let engine =
            engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::NO_PAD);

        // Decode the token using DecoderReader
        let mut decoder = read::DecoderReader::new(split_token[1].as_bytes(), &engine);
        let mut decoded_token = Vec::new();
        decoder.read_to_end(&mut decoded_token)?;

        let token_attributes: TokenAttributes = serde_json::from_slice(&decoded_token)?;
        let token = Token {
            raw_token,
            token_attributes,
        };
        Ok(token)
    }
}

// test
#[cfg(test)]
mod test {
    use super::*;

    // test Token::new()
    #[test]
    fn test_a_token() {
        // this token is an example token and already expired
        let raw_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJnZW4iOjM0MCwiZW5kcG9pbnQiOiJtcXR0LmRzaC1kZXYuZHNoLm5wLmF3cy5rcG4uY29tIiwiaXNzIjoiMCIsImNsYWltcyI6W3sicmVzb3VyY2UiOnsic3RyZWFtIjoiYWp1Y3B1YmxpYyIsInByZWZpeCI6Ii90dCIsInRvcGljIjoiYWp1Yy8jIiwidHlwZSI6InRvcGljIn0sImFjdGlvbiI6InN1YnNjcmliZSJ9XSwiZXhwIjoxNjY2Mjg0MTA0LCJwb3J0cyI6eyJtcXR0d3NzIjpbNDQzLDg0NDNdLCJtcXR0cyI6Wzg4ODNdfSwiY2xpZW50LWlkIjoiMmQzODE0ZWEtODQ5ZS00YjZlLWI0MzUtZjkyZDExZjhlMmY2IiwiaWF0IjoxNjY1NjgyOTA0LCJ0ZW5hbnQtaWQiOiJhanVjIn0.NFpVk7y4p5EeDRdPCpwlLrV0EW4JafpsUgij_Wu7ozM".to_string();
        let token = Token::new(raw_token.clone()).unwrap();

        let validation_token = Token {
            raw_token: raw_token.to_string(),
            token_attributes: TokenAttributes {
                gen: 340,
                endpoint: "mqtt.dsh-dev.dsh.np.aws.kpn.com".to_string(),
                iss: "0".to_string(),
                claims: vec![Claims {
                    resource: Resource {
                        stream: "ajucpublic".to_string(),
                        prefix: "/tt".to_string(),
                        topic: "ajuc/#".to_string(),
                        type_: None,
                    },
                    action: "subscribe".to_string(),
                }],
                exp: 1666284104,
                ports: Ports {
                    mqtts: vec![8883],
                    mqttwss: vec![443, 8443],
                },
                client_id: "2d3814ea-849e-4b6e-b435-f92d11f8e2f6".to_string(),
                iat: 1665682904,
                tenant_id: "ajuc".to_string(),
            },
        };

        // assert equal
        assert_eq!(validation_token, token);
    }
}
