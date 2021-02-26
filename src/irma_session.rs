use crate::config;
use crate::errors::Error;
use crate::errors::Error::ParseError;
use crate::irma::{Attribute, IrmaProofPayload, IrmaRequest, SessionResponse, SessionStatus};
use futures_core::Stream;
use futures_util::future::Ready;
use futures_util::{self, future, StreamExt};
use reqwest::header::CONTENT_TYPE;
use std::convert::TryFrom;

// abtraction over an IRMA authentication session
pub struct IrmaSession {
    pub qr: String,
    pub token: String,
}

impl IrmaSession {
    // create a new IRMA session
    pub async fn new() -> Result<IrmaSession, Error> {
        let attributes: Vec<Vec<Attribute>> =
            serde_json::from_str(&config::get("IRMA_ATTRIBUTES"))?;
        let disclosure = IrmaRequest::disclosure(vec![attributes]).as_jwt().await?;
        let url = format!("{}/session", config::get("IRMA_SERVER"));
        let client = reqwest::Client::new();

        info!("Starting IRMA session");
        let session_response: SessionResponse = client
            .post(&url)
            .header(CONTENT_TYPE, "text/plain")
            .body(disclosure)
            .send()
            .await?
            .json()
            .await?;
        info!("Started IRMA session: {}", &session_response.token);

        let qr = serde_json::to_string(&session_response.session_ptr)?;

        Ok(IrmaSession {
            token: session_response.token,
            qr,
        })
    }

    // cancel an IRMA session
    pub async fn stop(&self) -> Result<(), Error> {
        info!("Stopping IRMA session: {}", &self.token);
        let url = format!("{}/session/{}", config::get("IRMA_SERVER"), &self.token);
        let client = reqwest::Client::new();
        client.delete(&url).send().await?;

        Ok(())
    }

    // retrieve the proof for the current session
    pub async fn get_proof_payload(&self) -> Result<String, Error> {
        info!("Verify proof of IRMA session: {}", &self.token);
        let username = IrmaProofPayload::verify(&self.token).await?;

        Ok(username)
    }

    // subscribe to SSE for session updates
    pub async fn get_updates(&self) -> Result<impl Stream<Item = SessionStatus>, Error> {
        let url = format!(
            "{}/session/{}/statusevents",
            config::get("IRMA_SERVER"),
            &self.token
        );

        info!("Subscribing to SSE for IRMA session: {}", &self.token);
        let stream = reqwest::get(&url).await?.bytes_stream();

        // parse SSE to an IRMA session status
        let filtered_stream = stream.filter_map(|event| -> Ready<Option<SessionStatus>> {
            info!("Received SSE {:?}", event);
            let status = IrmaSession::parse_sse(event).expect("parse error");
            future::ready(Some(status))
        });

        Ok(filtered_stream)
    }

    // parse an IRMA SSE to an IRMA SessionStatus
    fn parse_sse(event: Result<bytes::Bytes, reqwest::Error>) -> Result<SessionStatus, Error> {
        let bytes =
            event.map_err(|e| ParseError(format!("Could not parse session status SSE {}", e)))?;

        let msg: String = String::from_utf8(bytes.to_vec())
            .map_err(|e| ParseError(format!("Could not parse session status SSE {}", e)))?;

        let mut parts = msg.splitn(2, ':');

        let value = match (parts.next(), parts.next()) {
            (Some("data"), Some(value)) => value.trim().trim_matches('"').to_string(),
            (Some("event"), Some(_)) => {
                // open events are translated to initial state
                return Ok(SessionStatus::Initialized);
            }
            _ => {
                error!("Could not parse session status SSE {}", &msg);
                return Err(ParseError("Could not parse session status SSE".to_string()));
            }
        };

        SessionStatus::try_from(value)
    }
}
