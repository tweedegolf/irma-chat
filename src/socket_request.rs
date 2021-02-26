use crate::errors::Error;
use std::fmt::Display;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct SocketRequest(Message);

impl Display for SocketRequest {
    // convert ws client message to string
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.0)
    }
}

impl SocketRequest {
    // create an abstraction over a ws client message
    pub fn from_message(
        message: Option<Result<Message, tungstenite::Error>>,
    ) -> Result<Self, Error> {
        Ok(SocketRequest(message.ok_or(Error::IgnorableError)??))
    }

    // request to start a new IRMA session
    pub fn is_start(&self) -> bool {
        self.0.to_string() == "start"
    }

    // request to stop the current authentication session
    pub fn is_stop(&self) -> bool {
        self.0.to_string() == "stop"
    }

    // message indicating the connetion was closed
    pub fn is_close(&self) -> bool {
        self.0.is_close()
    }
}
