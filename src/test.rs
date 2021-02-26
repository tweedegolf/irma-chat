use crate::jwt::{decode, encode, encode_rsa};
use crate::session_jwt::SessionJwt;
use crate::{auth_socket, chat_socket, config};
use chrono::Utc;
use dotenv::dotenv;

use crate::chat_socket::PeerMap;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::connect;

#[derive(Deserialize)]
struct Action {
    payload: String,
}

async fn init_session() -> mockito::Mock {
    dotenv().ok();
    env::set_var("IRMA_SERVER", &mockito::server_url());

    let start_mock = mockito::mock("POST", "/session")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"
                {
                    "sessionPtr": {
                        "u": "http://127.0.0.1:1234/irma/session/NLSNBLePEryjuZcsZ1Vg",
                        "irmaqr":"disclosing"
                    },
                    "token":"P9hCuu0hCQtfFndWXgoQ"
                }
            "#,
        )
        .create();

    tokio::spawn(async {
        let auth_host = config::get("WS_HOST");
        let auth_listener = TcpListener::bind(auth_host).await.expect("Failed to bind");
        while let Ok((stream, _)) = auth_listener.accept().await {
            tokio::spawn(auth_socket::handle_auth_connection(stream));
        }
    });

    start_mock
}

#[tokio::test(flavor = "multi_thread")]
async fn test_valid_session() {
    let start_mock = init_session().await;

    let sse_mock = mockito::mock("GET", "/session/P9hCuu0hCQtfFndWXgoQ/statusevents")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body_from_fn(|w| {
            w.write(b"data: CONNECTED\n\r")?;
            w.write(b"data: DONE\n\r")?;
            Ok(())
        })
        .create();

    let claim = json!({
      "attributes": {
        "pbdf.pbdf.idin.initials": "Foo",
        "pbdf.pbdf.idin.familyname": "Bar",
      },
      "exp": Utc::now().timestamp() + 300,
      "iat": Utc::now().timestamp(),
      "iss": "irmaserver",
      "status": "VALID",
      "sub": "disclosure_result"
    });

    let app_priv_key_file = config::get("IRMA_SERVER_JWT_PRIVKEY_FILE");
    let jwt = encode_rsa(app_priv_key_file, claim).await.unwrap();

    let proof_mock = mockito::mock("GET", "/session/P9hCuu0hCQtfFndWXgoQ/getproof")
        .with_status(200)
        .with_body(jwt)
        .create();

    let url = format!("ws://{}", env::var("WS_HOST").unwrap());
    let (mut socket, _) = connect(url).expect("Failed to connect");

    socket.write_message("start".into()).unwrap();

    let qr = r#"{"action":"qr","payload":"{\"u\":\"http://127.0.0.1:1234/irma/session/NLSNBLePEryjuZcsZ1Vg\",\"irmaqr\":\"disclosing\"}"}"#;
    assert_eq!(socket.read_message().unwrap().to_string(), qr);
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"status","payload":"CONNECTED"}"#
    );
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"status","payload":"DONE"}"#
    );

    let jwt_action: Action =
        serde_json::from_str(socket.read_message().unwrap().to_string().as_str()).unwrap();
    let key = config::get("APP_JWT_KEY");
    let decode_result = decode::<SessionJwt>(key, jwt_action.payload).unwrap();

    assert_eq!(decode_result.sub, "Foo Bar");

    start_mock.assert();
    sse_mock.assert();
    proof_mock.assert();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_invalid_session() {
    let start_mock = init_session().await;

    let sse_mock = mockito::mock("GET", "/session/P9hCuu0hCQtfFndWXgoQ/statusevents")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body_from_fn(|w| {
            w.write(b"data: CONNECTED\n\r")?;
            w.write(b"data: DONE\n\r")?;
            Ok(())
        })
        .create();

    let app_priv_key_file = config::get("IRMA_SERVER_JWT_PRIVKEY_FILE");
    let claim = json!({
      "attributes": {
        "pbdf.pbdf.idin.initials": "Foo",
        "pbdf.pbdf.idin.familyname": "Bar",
      },
      "exp": Utc::now().timestamp() + 300,
      "iat": Utc::now().timestamp(),
      "iss": "irmaserver",
      "status": "INVALID",
      "sub": "disclosure_result"
    });
    let jwt = encode_rsa(app_priv_key_file, claim).await.unwrap();

    let proof_mock = mockito::mock("GET", "/session/P9hCuu0hCQtfFndWXgoQ/getproof")
        .with_status(200)
        .with_body(jwt)
        .create();

    let url = format!("ws://{}", env::var("WS_HOST").unwrap());
    let (mut socket, _) = connect(url).expect("Failed to connect");

    socket.write_message("start".into()).unwrap();

    let qr = r#"{"action":"qr","payload":"{\"u\":\"http://127.0.0.1:1234/irma/session/NLSNBLePEryjuZcsZ1Vg\",\"irmaqr\":\"disclosing\"}"}"#;
    assert_eq!(socket.read_message().unwrap().to_string(), qr);
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"status","payload":"CONNECTED"}"#
    );
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"status","payload":"DONE"}"#
    );
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"error","payload":"Could not verify claim"}"#
    );

    start_mock.assert();
    sse_mock.assert();
    proof_mock.assert();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_cancel() {
    let start_mock = init_session().await;

    let sse_mock = mockito::mock("GET", "/session/P9hCuu0hCQtfFndWXgoQ/statusevents")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body_from_fn(|w| {
            w.write(b"data: CONNECTED\n\r")?;
            w.write(b"data: CANCELLED\n\r")?;
            Ok(())
        })
        .create();

    let url = format!("ws://{}", env::var("WS_HOST").unwrap());
    let (mut socket, _) = connect(url).expect("Failed to connect");

    socket.write_message("start".into()).unwrap();

    let qr = r#"{"action":"qr","payload":"{\"u\":\"http://127.0.0.1:1234/irma/session/NLSNBLePEryjuZcsZ1Vg\",\"irmaqr\":\"disclosing\"}"}"#;
    assert_eq!(socket.read_message().unwrap().to_string(), qr);
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"status","payload":"CONNECTED"}"#
    );
    assert_eq!(
        socket.read_message().unwrap().to_string(),
        r#"{"action":"status","payload":"CANCELLED"}"#
    );

    start_mock.assert();
    sse_mock.assert();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_chat() {
    dotenv().ok();

    tokio::spawn(async {
        let chat_host = config::get("CHAT_WS_HOST");
        let chat_listener = TcpListener::bind(chat_host).await.expect("Failed to bind");
        let state = PeerMap::new(Mutex::new(HashMap::new()));

        while let Ok((stream, addr)) = chat_listener.accept().await {
            tokio::spawn(chat_socket::handle_chat_connection(
                state.clone(),
                stream,
                addr,
            ));
        }
    });

    let app_key = config::get("APP_JWT_KEY");
    let claim = json!({
      "exp": Utc::now().timestamp() + 300,
      "sub": "Foo Bar"
    });
    let jwt = encode(app_key, claim).unwrap();

    let url = format!("ws://{}", env::var("CHAT_WS_HOST").unwrap());
    let (mut socket, _) = connect(url).expect("Failed to connect");

    socket.write_message(jwt.into()).unwrap();
    socket.write_message("Hello World!".into()).unwrap();

    let time = Utc::now().timestamp();

    assert_eq!(
        socket.read_message().unwrap().to_string(),
        format!(
            "{{\"user\":\"Foo Bar\",\"time\":{},\"its_me\":true,\"msg\":null}}",
            time
        )
    );

    assert_eq!(
        socket.read_message().unwrap().to_string(),
        format!(
            "{{\"user\":\"Foo Bar\",\"time\":{},\"its_me\":true,\"msg\":\"Hello World!\"}}",
            time
        )
    );

    socket.close(None).unwrap();
}
