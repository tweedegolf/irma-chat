mod auth_socket;
mod chat_socket;
mod config;
mod errors;
mod irma;
mod irma_session;
mod jwt;
mod session_jwt;
mod socket_request;
mod socket_response;

#[macro_use]
extern crate log;

use crate::chat_socket::PeerMap;
use dotenv::dotenv;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::net::TcpListener;

// bind the chat and the authentication websocket to the provides ports
pub async fn serve() {
    let auth_host = config::get("WS_HOST");
    let chat_host = config::get("CHAT_WS_HOST");

    info!("Starting WS server on {} and {}", auth_host, chat_host);

    let auth_listener = TcpListener::bind(auth_host).await.expect("Failed to bind");
    let chat_listener = TcpListener::bind(chat_host).await.expect("Failed to bind");

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // accept new ws connections and stop when receiving an interrupt
    loop {
        tokio::select! {
            auth_stream = auth_listener.accept() => if let Ok((stream, _)) = auth_stream {
                tokio::spawn(auth_socket::handle_auth_connection(stream));
            },
            chat_stream = chat_listener.accept() => if let Ok((stream, addr)) = chat_stream {
                tokio::spawn(chat_socket::handle_chat_connection(state.clone(), stream, addr));
            },
            _ = tokio::signal::ctrl_c() => break,
        }
    }
}

// application entry point
#[tokio::main]
pub async fn main() {
    dotenv().ok();
    env_logger::init();
    serve().await;
}

#[cfg(test)]
mod test;
