import { writable } from 'svelte/store';

export enum SessionStatus {
  INITIALIZED = 'INITIALIZED',
  CONNECTED = 'CONNECTED',
  CANCELLED = 'CANCELLED',
  DONE = 'DONE',
  TIMEOUT = 'TIMEOUT',
}

export enum ActionType {
  qr = 'qr',
  status = 'status',
  jwt = 'jwt',
}

interface QRResponse {
  action: ActionType.qr,
  payload: string,
}

interface StatusResponse {
  action: ActionType.status,
  payload: SessionStatus,
}

interface JWTResponse {
  action: ActionType.jwt,
  payload: string,
}

type SessionResponse = QRResponse | StatusResponse | JWTResponse;

interface SessionState {
  status: null | SessionStatus,
  qrCode: null | string,
  jwt: null | string,
}

/**
 * Represents a disclosure session
 */
class Session {

  private status: SessionStatus = null;
  private qrCode: string = null;
  private jwt: string = null;
  private socket: WebSocket = null;
  private store = writable(this.getState());

  public getState(): SessionState {
    return {
        status: this.status,
        qrCode: this.qrCode,
        jwt: this.jwt,
    };
  }

  private async connect() {
    if (
      !this.socket ||
      this.socket.readyState === WebSocket.CLOSED ||
      this.socket.readyState === WebSocket.CLOSING
    ) {
      const host = window.location.host;
      this.socket = new WebSocket(`wss://${host}/auth`);
      return new Promise((resolve) => this.socket.addEventListener('open', resolve));
    }
  }

  private handleMessage(e: MessageEvent) {
    let response: SessionResponse;

    try {
      response = JSON.parse(e.data);
    } catch(e) {
      console.warn('Invalid server message', e);
      return;
    }

    switch (response.action) {
      case ActionType.qr:
        this.qrCode = response.payload;
        break;
      case ActionType.status:
        this.status = response.payload;
        break;
      case ActionType.jwt:
        this.jwt = response.payload;
        break;
    }

    this.store.set(this.getState());
  }

  public getStore() {
    return this.store;
  }

  public reset() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send('stop');
      this.socket.close();
    }
    this.status = null;
    this.qrCode = null;
    this.jwt = null;

    this.store.set(this.getState());
  }

  public async start() {
    this.status = SessionStatus.INITIALIZED;
    this.store.set(this.getState());

    await this.connect();
    this.socket.send('start');
    this.socket.addEventListener('message', this.handleMessage.bind(this));
  }
}

const session = new Session();

export default session;
