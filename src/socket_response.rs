use crate::errors::Error;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct SocketResponse {
    action: &'static str,
    payload: String,
}

impl SocketResponse {
    const ACTION_QR: &'static str = "qr";
    const ACTION_STATUS: &'static str = "status";
    const ACTION_JWT: &'static str = "jwt";
    const ACTION_ERROR: &'static str = "error";

    // message used to show a IRMA QR code or forward the user to the IRMA app directly
    pub fn qr(qr: String) -> Self {
        SocketResponse {
            action: SocketResponse::ACTION_QR,
            payload: qr,
        }
    }

    // session status update
    pub fn status(status: String) -> SocketResponse {
        SocketResponse {
            action: SocketResponse::ACTION_STATUS,
            payload: status,
        }
    }

    // session error
    pub fn error(error: String) -> SocketResponse {
        SocketResponse {
            action: SocketResponse::ACTION_ERROR,
            payload: error,
        }
    }

    // resulting session JWT
    pub fn jwt(jwt: String) -> SocketResponse {
        SocketResponse {
            action: SocketResponse::ACTION_JWT,
            payload: jwt,
        }
    }

    // encode a response as (json) string
    pub fn encode(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
