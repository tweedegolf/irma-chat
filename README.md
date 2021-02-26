# IRMA chat

A chat application, with some substantiated guarantees of knowing who you are chatting with.
Uses [IRMA](https://irma.app/) to provide your real name.

## Demo

You can try the DEMO at https://irma-chat.tweede.golf/ - make sure to have your IRMA app installed and configured.

![demo](https://github.com/tweedegolf/irma-chat/blob/master/demo.png?raw=true)

## Goal

This project was made to better learn the [Rust](https://www.rust-lang.org/) programming language,
to get some hands-on knowledge of IRMA and to learn the frontend framework [Svelte](https://svelte.dev/).
Because learning a new frontend framework was one of the goal the [IRMA frontend packages](https://irma.app/docs/irma-frontend/) were not used.
In any production application I would recommand using these packages.

## Technical overview

The application consist out of three components:

- A frontend SPA, written in Typescript / Svelte that guides the user trough the authentication flow and shows the chat interface
- A backend application written in Rust that only comminucates over websockets. There are two websocket endpoints, one for the authentication flow with IRMA and one to send and receive chat messages
- An [IRMA server](https://irma.app/docs/irma-server/) to perform IRMA sessions

Session status updates are sent from IRMA server to the Rust backend over [Server Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) and are forwarded to the client via the esteblished websocket.

## Usage

Deployment of this application is specific to the platform were you want to host the application.
The DEMO application is hosted on a Kubernetes cluster.

Running the applications consist of the following steps:

- Transpile the frontend using rollup and serve `index.html`, javascript files and stylesheets as static files
- Compile the rust code and run the binary configures with environment variables
- Startup and configure the IRMA server

## Configuration

The following environment variables a needed by the backend:

```
WS_HOST: address and port to listen to for authentication sessions, for example: "0.0.0.0:9090"
CHAT_WS_HOST: address and port to listen to for chat sessions
APP_JWT_KEY: a secret key
APP_JWT_PRIVKEY_FILE: filename of a RS256 private key
APP_JWT_PUBKEY_FILE: filename of a RS256 public key
IRMA_SERVER: (internal) address of the IRMA server
IRMA_SERVER_JWT_PUBKEY_FILE: filename the IRMA a RS256 public key
IRMA_ATTRIBUTES: attributes to use as username during chat i.e.: '[["pbdf.gemeente.personalData.fullname"], ["pbdf.pbdf.idin.initials", "pbdf.pbdf.idin.familyname"]]'
APP_NAME: name of the application
```

In addition the IRMA server could be configured using the following:

```
IRMASERVER_URL: publicly accessible endpoint for the IRMA server
IRMASERVER_SSE: whould be "true"
IRMASERVER_JWT_PRIVKEY_FILE: filename of the IRMA RS256 private key
IRMASERVER_EMAIL: your email
IRMASERVER_NO_AUTH: should be "false"
IRMASERVER_REQUESTORS: name and authentication method for your app - see the IRMA server documentation
```

## Generate keys 

JWT encoded messages are used between the IRMA server nd the backend.
This is not struicly needed when the connection between the akend en the IRMA server is secured, but it provides "defense in depth". 

Keys can be generated with the following commands:

```
ssh-keygen -t rsa -b 4096 -m PEM -f app_jwtRS256.key
# Don't add passphrase
openssl rsa -in app_jwtRS256.key -pubout -outform PEM -out app_jwtRS256.key.pub
```

## Tests

The are functional tests for the main parts of this application.
You can run them uzing `cargo test`. Make sure you have generated the
JWT keys and configured all settings using the environment or a `.env` file.
