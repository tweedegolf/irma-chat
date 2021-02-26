use crate::errors::Error;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use crate::config;
use crate::jwt::decode;
use crate::session_jwt::SessionJwt;
use crate::socket_request::SocketRequest;
use chrono::Utc;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{self, future, pin_mut, stream::TryStreamExt, SinkExt, StreamExt};
use log::error;
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug)]
pub struct ChatClient {
    user: String,
    tx: UnboundedSender<Message>,
}

#[derive(Serialize, Debug, Clone)]
struct ChatMessage {
    user: String,
    time: i64,
    its_me: bool,
    msg: Option<String>,
}

impl ChatMessage {
    // encode a chat message to a ws message (as json)
    fn to_message(&self) -> Message {
        let encoded_chat_message = serde_json::to_string(self).unwrap();

        encoded_chat_message.into()
    }
}

// in memory administration of connected clients
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, ChatClient>>>;

// handle a chat session
async fn accept_chat_connection(
    peer_map: PeerMap,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Error> {
    info!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    info!("WS connection established: {}", addr);
    let token = SocketRequest::from_message(read.next().await)?;

    info!("Received message token: {:?}", &token);
    let key = config::get("APP_JWT_KEY");
    let jwt = match decode::<SessionJwt>(key, token.to_string()) {
        Ok(jwt) => {
            info!("Found valid JWT token");
            jwt
        }
        Err(error) => {
            let msg = json!({ "error": format!("Authentication error: {:?}", error) }).to_string();
            write.send(msg.into()).await?;

            return Err(Error::InvalidJWT);
        }
    };

    // add the new client to the peer administration
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(
        addr,
        ChatClient {
            user: jwt.sub.clone(),
            tx,
        },
    );

    // broadcast that a new client connected
    for (peer_addr, recipient) in peer_map.lock().unwrap().iter() {
        let chat_msg = ChatMessage {
            user: jwt.sub.clone(),
            time: Utc::now().timestamp(),
            its_me: peer_addr == &addr,
            msg: None,
        };
        recipient.tx.unbounded_send(chat_msg.to_message()).unwrap();
    }

    // forward all messages to all connected peers
    let broadcast_incoming = read.try_for_each(|msg| {
        info!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        for (peer_addr, recipient) in peer_map.lock().unwrap().iter() {
            let chat_msg = ChatMessage {
                user: jwt.sub.clone(),
                time: Utc::now().timestamp(),
                its_me: peer_addr == &addr,
                msg: Some(msg.to_string()),
            };
            recipient.tx.unbounded_send(chat_msg.to_message()).unwrap();
        }

        future::ok(())
    });

    // message plumbing, forward all incoming messages
    let receive_from_others = rx.map(Ok).forward(write);
    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    // when a client diconnects, remove them from the administration
    info!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
    Ok(())
}

// handle a chat ws connection and print blocking errors
pub async fn handle_chat_connection(peer_map: PeerMap, stream: TcpStream, addr: SocketAddr) {
    if let Err(e) = accept_chat_connection(peer_map, stream, addr).await {
        error!("Error {:?}", e);
    }
}
