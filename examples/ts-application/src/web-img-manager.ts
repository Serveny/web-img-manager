/**
 * @fileOverview Bindings for web img manager
 * @author Serveny
 * @version 1.0.0
 */

type LobbyId = string;
type RoomId = number;
type ImgId = string;

/** Class for communication with web img manager server. */
export class WebImgManager {
  notifications: Notifications | null = null;

  constructor(public server_addr: string) {}

  async get_room_img_list(lobby_id: LobbyId, room_id: RoomId) {
    return send(
      `http://${this.server_addr}/list/${lobby_id}/${room_id}`,
      'GET'
    );
  }

  async upload_img(lobby_id: LobbyId, room_id: RoomId, image: File) {
    const url = `http://${this.server_addr}/upload/${lobby_id}/${room_id}`;
    const formData = new FormData();
    formData.append('image', image);

    return await fetch(url, {
      method: 'POST',
      body: formData,
    });
  }

  async delete(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId) {
    let url = `http://${this.server_addr}/delete/${lobby_id}`;
    if (room_id) url += `/${room_id}`;
    if (img_id) url += `/${img_id}`;
    return send(url, 'POST');
  }

  async sendChatMessage(lobby_id: LobbyId, msg: string) {
    let url = `http://${this.server_addr}/chat`;
    return send(url, 'POST', { lobby_id, msg });
  }

  connect(lobby_id: LobbyId) {
    this.notifications = new Notifications(this.server_addr, lobby_id);
    return this;
  }
}

/** Class for communication with web img manager web socket server. */
class Notifications {
  emitter = new EventEmitter();

  constructor(server_addr: string, lobby_id: LobbyId) {
    const socket = new WebSocket(`ws://${server_addr}/ws/${lobby_id}`);

    // Declare Events
    socket.addEventListener('open', (event) => {
      this.emitter.emit('Connected', event);
    });

    socket.addEventListener('close', (event) => {
      this.emitter.emit('Disconnected', event);
    });

    socket.addEventListener('error', (event) => {
      this.emitter.emit('Error', event);
    });

    socket.addEventListener('message', (event) => {
      const evData = JSON.parse(event.data);
      this.emitter.emit(evData.event, evData);
    });
  }

  onConnected(handler: (ev: Event) => void) {
    this.emitter.on('Connected', handler);
  }

  onDisconnected(handler: (ev: Event) => void) {
    this.emitter.on('Disonnected', handler);
  }

  onError(handler: (ev: Event) => void) {
    this.emitter.on('Error', handler);
  }

  onImageUploaded(handler) {
    this.emitter.on('ImageUploaded', handler);
  }

  onLobbyDeleted(handler) {
    this.emitter.on('LobbyDeleted', handler);
  }

  onRoomDeleted(handler) {
    this.emitter.on('RoomDeleted', handler);
  }

  onImageDeleted(handler) {
    this.emitter.on('ImageDeleted', handler);
  }

  onChatMessage(handler) {
    this.emitter.on('ChatMessage', handler);
  }
}

async function send(url: string, method: string, params?: object) {
  return fetch(url, {
    method: method,
    headers: {
      'content-Type': 'application/json',
    },
    body: JSON.stringify(params),
  }).then((response) => {
    if (!response.ok) {
      const error = new Error(
        `Response error: ${response.status} - ${response.statusText}`
      );
      console.error(error);
      throw error;
    }
    return response.json();
  });
}

class Listener<Ev> {
  constructor(public name: string, public callback: (ev: Ev) => void) {}
}

/** Helper Class for event emitting */
class EventEmitter {
  listeners: Listener<Event>[] = [];

  emit(eventName: string, event: Event) {
    this.listeners
      .filter(({ name }) => name === eventName)
      .forEach(({ callback }) => callback(event), 0);
  }

  on(name: string, callback: (ev: Event) => void) {
    if (typeof callback === 'function' && typeof name === 'string') {
      this.listeners.push({ name, callback });
    }
  }

  off(eventName: string, callback: (ev: Event) => void) {
    this.listeners = this.listeners.filter(
      (listener) =>
        !(listener.name === eventName && listener.callback === callback)
    );
  }

  destroy() {
    this.listeners.length = 0;
  }
}
