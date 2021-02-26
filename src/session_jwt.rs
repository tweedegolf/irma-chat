use crate::config;
use crate::errors::Error;
use crate::jwt::encode;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionJwt {
    pub exp: i64,
    pub sub: String,
}

impl SessionJwt {
    // create a new chat application session clains
    pub fn new(sub: String) -> Self {
        SessionJwt {
            exp: Utc::now().timestamp() + 3600,
            sub,
        }
    }

    // encode and sign claims as a JWT
    pub fn as_jwt(&self) -> Result<String, Error> {
        let key = config::get("APP_JWT_KEY");
        let jwt = encode(key, self)?;

        Ok(jwt)
    }
}
