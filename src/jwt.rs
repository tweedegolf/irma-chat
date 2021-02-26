use crate::config;
use crate::errors::Error;
use jsonwebtoken::{self, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn read_key(key_file: String) -> Result<Vec<u8>, Error> {
    let mut file = File::open(&key_file)
        .await
        .map_err(|_| Error::InvalidJWTKey)?;

    let mut key = Vec::new();
    file.read_to_end(&mut key)
        .await
        .map_err(|_| Error::InvalidJWTKey)?;

    let key = key;

    Ok(key)
}

// encode and sign a JWT using RS256
pub async fn encode_rsa<T: Serialize>(key_file: String, claim: T) -> Result<String, Error> {
    let key = read_key(key_file).await?;
    let kid = config::get("APP_NAME");

    jsonwebtoken::encode(
        &Header {
            typ: Some("JWT".to_string()),
            alg: Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        },
        &claim,
        &EncodingKey::from_rsa_pem(key.as_ref())?,
    )
    .map_err(Error::JWTError)
}

// decode and verify a JWT using RS256
pub async fn decode_rsa<T: DeserializeOwned>(key_file: String, token: String) -> Result<T, Error> {
    let key = read_key(key_file).await?;

    match jsonwebtoken::decode::<T>(
        &token,
        &DecodingKey::from_rsa_pem(&key)?,
        &Validation::new(Algorithm::RS256),
    ) {
        Ok(result) => Ok(result.claims),
        Err(e) => Err(Error::JWTError(e)),
    }
}

// encode and sign a JWT using HS256
pub fn encode<T: Serialize>(key: String, claim: T) -> Result<String, Error> {
    jsonwebtoken::encode(
        &Header {
            typ: Some("JWT".to_string()),
            alg: Algorithm::HS256,
            ..Default::default()
        },
        &claim,
        &EncodingKey::from_secret(key.as_ref()),
    )
    .map_err(Error::JWTError)
}

// decode and verify a JWT using HS256
pub fn decode<T: DeserializeOwned>(key: String, token: String) -> Result<T, Error> {
    match jsonwebtoken::decode::<T>(
        &token,
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(result) => Ok(result.claims),
        Err(e) => Err(Error::JWTError(e)),
    }
}
