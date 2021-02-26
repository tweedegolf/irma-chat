use tokio_tungstenite::tungstenite;

// wrap errors from third party libraries in our own error type
#[derive(Debug)]
pub enum Error {
    IgnorableError,
    InvalidJWT,
    InvalidJWTKey,
    InvalidProofStatus,
    EnvironmentError(std::env::VarError),
    ParseError(String),
    SerializationError(serde_json::Error),
    RequestError(reqwest::Error),
    WebsocketError(tungstenite::Error),
    JWTError(jsonwebtoken::errors::Error),
    IoError(std::io::Error),
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::ParseError(err)
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Self::EnvironmentError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err)
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::WebsocketError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::JWTError(err)
    }
}
