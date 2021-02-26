use crate::config;
use crate::errors::Error;
use crate::jwt::{decode_rsa, encode_rsa};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpecificAttribute {
    #[serde(rename = "type")]
    attribute_type: String,
    value: Option<String>,
    not_null: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Attribute {
    Simple(String),
    _Specific(SpecificAttribute),
}

pub type ConDisCon = Vec<Vec<Vec<Attribute>>>;

#[derive(Serialize, Debug, Clone)]
pub struct IrmaRequest {
    #[serde(rename = "@context")]
    context: &'static str,
    disclose: ConDisCon,
}

#[derive(Serialize, Debug, Clone)]
struct ExtendedIrmaRequest {
    validity: u64,
    timeout: u64,
    request: IrmaRequest,
}

#[derive(Serialize, Debug, Clone)]
struct IrmaJwt {
    iat: i64,
    sub: &'static str,
    sprequest: ExtendedIrmaRequest,
}

impl IrmaJwt {
    const DISCLOSURE: &'static str = "verification_request";
}

// abstraction over a request to the IRMA server
impl IrmaRequest {
    const DISCLOSURE: &'static str = "https://irma.app/ld/request/disclosure/v2";

    // create a discloure requests
    pub fn disclosure(cdc: ConDisCon) -> Self {
        IrmaRequest {
            context: Self::DISCLOSURE,
            disclose: cdc,
        }
    }

    // encode and sign a request to the IRMA server
    pub async fn as_jwt(&self) -> Result<String, Error> {
        let claim = IrmaJwt {
            iat: Utc::now().timestamp(),
            sub: IrmaJwt::DISCLOSURE,
            sprequest: ExtendedIrmaRequest {
                validity: 300,
                timeout: 300,
                request: self.to_owned(),
            },
        };

        let app_priv_key_file = config::get("APP_JWT_PRIVKEY_FILE");

        // https://irma.app/docs/irma-server/#requestor-authentication
        encode_rsa(app_priv_key_file, claim).await
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum SessionType {
    Disclosing,
    Signing,
    Issuing,
}

#[derive(Deserialize, Serialize)]
pub struct SessionPointer {
    u: String,
    #[serde(rename = "irmaqr")]
    irma_qr: SessionType,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub token: String,
    pub session_ptr: SessionPointer,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofStatus {
    Valid,
    Invalid,
    InvalidTimestamp,
    UnmatchedRequest,
    MissingAttributes,
    Expired,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionStatus {
    Initialized,
    Connected,
    Cancelled,
    Done,
    Timeout,
}

impl fmt::Display for SessionStatus {
    // covert an IRMA SessionStatus to a String
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            SessionStatus::Initialized => "INITIALIZED",
            SessionStatus::Connected => "CONNECTED",
            SessionStatus::Cancelled => "CANCELLED",
            SessionStatus::Done => "DONE",
            SessionStatus::Timeout => "TIMEOUT",
        };

        write!(f, "{}", status)
    }
}

impl TryFrom<String> for SessionStatus {
    type Error = Error;

    // parse a string to an IRMA SessionStatus
    fn try_from(status: String) -> Result<Self, Error> {
        let session_status = match status.as_str() {
            "INITIALIZED" => SessionStatus::Initialized,
            "CONNECTED" => SessionStatus::Connected,
            "CANCELLED" => SessionStatus::Cancelled,
            "DONE" => SessionStatus::Done,
            "TIMEOUT" => SessionStatus::Timeout,
            _ => {
                return Err(Error::ParseError(format!(
                    "Could not parse SSE to sessions status '{}'",
                    &status
                )))
            }
        };

        Ok(session_status)
    }
}

type IrmaAttributes = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug)]
pub struct IrmaProofPayload {
    attributes: IrmaAttributes,
    exp: i64,
    iat: i64,
    iss: String,
    status: ProofStatus,
    sub: String,
}

impl IrmaProofPayload {
    // request and verify the proof for an IRMA session
    pub async fn verify(token: &str) -> Result<String, Error> {
        // https://irma.app/docs/api-irma-server/#get-session-token-result-jwt
        let url = format!("{}/session/{}/getproof", config::get("IRMA_SERVER"), token);
        let response: String = reqwest::get(&url).await?.text().await?;

        info!("Retrieved proof from the IRMA server for session {}", token);

        let irma_pub_key_file = config::get("IRMA_SERVER_JWT_PUBKEY_FILE");
        let token_data = decode_rsa::<IrmaProofPayload>(irma_pub_key_file, response).await?;

        // https://irma.app/docs/irma-server/#requestor-authentication
        if token_data.status != ProofStatus::Valid {
            return Err(Error::InvalidProofStatus);
        }

        let attributes: Vec<Vec<String>> = serde_json::from_str(&config::get("IRMA_ATTRIBUTES"))?;
        let mut value: Vec<String> = vec![];

        for attributes in attributes.into_iter() {
            for attribute in attributes {
                if let Some(part) = token_data.attributes.get(&attribute) {
                    value.push(part.to_string());
                }
            }
        }

        Ok(value.join(" "))
    }
}
