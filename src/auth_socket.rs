use crate::errors::Error;
use crate::irma::SessionStatus;
use crate::irma_session::IrmaSession;
use crate::session_jwt::SessionJwt;
use crate::socket_request::SocketRequest;
use crate::socket_response::SocketResponse;
use futures_core::stream::Stream;
use futures_util::future::{self, Ready};
use futures_util::stream::SplitSink;
use futures_util::{self, SinkExt, StreamExt};
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

// abstraction over a client ws
struct AuthSession {
    write: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub read: Pin<Box<dyn Send + Stream<Item = SocketRequest>>>,
}

impl AuthSession {
    // covert ws requests to instances of a SocketRequest
    pub async fn new(stream: TcpStream) -> Result<AuthSession, Error> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (write, read) = ws_stream.split();
        let read = read.filter_map(|message| -> Ready<Option<SocketRequest>> {
            info!("Received ws message from the client: {:?}", message);
            future::ready(SocketRequest::from_message(message.into()).ok())
        });

        Ok(AuthSession {
            write,
            read: Box::pin(read),
        })
    }

    // send a SocketResponse back to the client
    pub async fn send(&mut self, response: SocketResponse) -> Result<(), Error> {
        info!("Sending ws message to the client: {:?}", response);
        self.write.send(response.encode()?.into()).await?;

        Ok(())
    }
}

// verify the proof at the end of an IRMA session and send the resulting proof JWT
async fn finish_session(
    irma_session: &IrmaSession,
    auth_session: &mut AuthSession,
) -> Result<(), Error> {
    // retrieve a username from a IRMA proof
    let username = match irma_session.get_proof_payload().await {
        Ok(claim) => claim,
        Err(e) => {
            error!("Could not verify claim: {:?}", e);
            let error = SocketResponse::error("Could not verify claim".to_string());
            auth_session.send(error).await?;

            return Err(e);
        }
    };

    // create a application signed JWT containing the username for chat
    let jwt = SessionJwt::new(username).as_jwt()?;
    let action = SocketResponse::jwt(jwt);
    auth_session.send(action).await?;

    Ok(())
}

// handle new authentication ws connections
async fn accept_auth_connection(stream: TcpStream) -> Result<(), Error> {
    info!("New ws auth connection");
    let mut auth_session = AuthSession::new(stream).await?;
    let msg = auth_session.read.next().await;

    // we expect the first message to be 'start' (to start an IRMA session)
    if msg.is_none() || !msg.unwrap().is_start() {
        error!("Expected the first message to be 'start'");
        return Ok(());
    }

    // create a new IRMA session
    info!("Starting new IRMA session");
    let irma_session = IrmaSession::new().await?;
    let qr = SocketResponse::qr(irma_session.qr.clone());
    auth_session.send(qr).await?;

    // subscribe to updates from the IRMA server
    let mut upstream = irma_session.get_updates().await?;

    // wait for either updates from the IRMA server of messages from the client
    loop {
        tokio::select! {
            Some(request) = auth_session.read.next() => {
                // stop the session when a connection is closed a a session in canceled
                if request.is_close() || request.is_stop() {
                    warn!("Authentication session canceled");
                    irma_session.stop().await?;
                    break;
                }
            },
            Some(status) = upstream.next() => {
                info!("Received IRMA status update {}", status);

                // forward server updates to the client
                let action = SocketResponse::status(status.to_string());
                auth_session.send(action).await?;

                if status == SessionStatus::Cancelled || status == SessionStatus::Timeout {
                    warn!("Authentication session canceled or timed out");
                    break;
                }

                if status == SessionStatus::Done {
                    info!("Authentication session done, sending JWT");
                    finish_session(&irma_session, &mut auth_session).await?;
                    break;
                }
            }
            else => break,
        }
    }

    info!("Finished authentication session");
    Ok(())
}

// handle a ws connection, print possible errors
pub async fn handle_auth_connection(stream: TcpStream) {
    if let Err(e) = accept_auth_connection(stream).await {
        error!("Error during authentication session: {:?}", e);
    }
}
