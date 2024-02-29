/**
 * @fileOverview Bindings for web img manager
 * @author Serveny
 * @version 1.0.0
 */

/** Class for communication with web img manager server. */
class WebImgManager {
  /**
   * Creates class for communication with web img manager server.
   * @constructor
   * @param {string} server_addr - The address of the web img manager server.
   */
  constructor(server_addr) {
    this.server_addr = server_addr;
  }

  /**
   * Get img_id's for room.
   * @param {string} lobby_id - Uuid v4 lobby identificator
   * @param {string} room_id - Uuid v4 room identificator
   * @return {Promise<number[]>} List of img_id's
   * @throws {Error} response error
   */
  async get_room_img_list(lobby_id, room_id) {
    return send(
      `http://${this.server_addr}/list/${lobby_id}/${room_id}`,
      'GET'
    );
  }

  /**
   * Uploads the given image
   * @param {string} lobby_id - Uuid v4 lobby identificator
   * @param {string} room_id - Uuid v4 room identificator
   * @param {string} image - base64 encoded image string
   * @return {Promise<number>} int32 img_id
   * @throws {Error} response error
   */
  async upload_img(lobby_id, room_id, image) {
    return send(`http://${server_addr}/upload`, 'POST', {
      lobby_id,
      room_id,
      image,
    });
  }

  /**
   * Deletes on server room folder, lobby folder or thumb and big image
   * @param {string} lobby_id - Uuid v4 lobby identificator
   * @param {string|undefined} room_id - Uuid v4 room identificator
   * @param {string|undefined} img_id - int32 image identifier
   * @return {Promise<void>}
   * @throws {Error} response error
   */
  async delete(lobbyId, roomId, imgName) {
    let url = `http://${server_addr}/delete/${lobbyId}`;
    if (roomId) url += `/${roomId}`;
    if (imgName) url += `/${imgName}`;
    return send(url, 'POST');
  }

  /**
   * Sends chat message to server
   * @param {string} lobby_id - Uuid v4 lobby identificator
   * @param {string} msg- Chat message
   * @return {Promise<void>}
   * @throws {Error} response error
   */
  async sendChatMessage(lobby_id, msg) {
    let url = `http://${server_addr}/chat`;
    return send(url, 'POST', { lobby_id, msg });
  }

  /**
   * Connects to web img manager web socket server and register events
   * @param {string} lobby_id - Uuid v4 lobby identificator
   * @return {WebImgManager}
   * @throws {Error} response error
   */
  connect(lobby_id) {
    this.notifications = new Notifications(this.server_addr, lobby_id);
    return this;
  }
}

/** Class for communication with web img manager web socket server. */
class Notifications {
  /**
   * Creates class for communication with web img manager web socket server.
   * @constructor
   * @param {string} server_addr - The address of the web img manager server.
   * @param {string} lobby_id - Uuid v4 lobby identificator
   */
  constructor(server_addr, lobby_id) {
    const socket = new WebSocket(`ws://${server_addr}/ws/${lobby_id}`);

    // Declare Events
    this.emitter = new EventEmitter();

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

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onConnected(handler) {
    this.emitter.on('Connected', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onDisconnected(handler) {
    this.emitter.on('Disonnected', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onError(handler) {
    this.emitter.on('Error', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onImageUploaded(handler) {
    this.emitter.on('ImageUploaded', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onLobbyDeleted(handler) {
    this.emitter.on('LobbyDeleted', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onRoomDeleted(handler) {
    this.emitter.on('RoomDeleted', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onImageDeleted(handler) {
    this.emitter.on('ImageDeleted', handler);
  }

  /**
   * Register event listener
   * @param {function} handler - event handler
   * @return {void} request result as object parsed from JSON
   */
  onChatMessage(handler) {
    this.emitter.on('ChatMessage', handler);
  }
}

/**
 * Helper function: Sends request
 * @param {string} url - url to send to
 * @param {string} method - Request method (GET, POST, ...)
 * @param {object} params - Parameters object to send
 * @return {Promis<object>} request result as object parsed from JSON
 * @throws {Error} response error
 */
async function send(url, method, params) {
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

/** Helper Class for event emitting */
class EventEmitter {
  listeners = [];

  /**
   * emits event
   * @param {string} eventName - Name of the event
   * @param {object} event - Data object returned by the event
   * @return {void}
   */
  emit(eventName, event) {
    this.listeners
      .filter(({ name }) => name === eventName)
      .forEach(({ callback }) => callback(event), 0);
  }

  /**
   * Adds event listener
   * @param {string} eventName - Name of the event
   * @param {function} callback - Event handler
   * @return {void}
   */
  on(name, callback) {
    if (typeof callback === 'function' && typeof name === 'string') {
      this.listeners.push({ name, callback });
    }
  }

  /**
   * Removes event listener
   * @param {string} eventName - Name of the event
   * @param {function} callback - Event handler
   * @return {void}
   */
  off(eventName, callback) {
    this.listeners = this.listeners.filter(
      (listener) =>
        !(listener.name === eventName && listener.callback === callback)
    );
  }

  /**
   * Removes all event listener
   * @return {void}
   */
  destroy() {
    this.listener.length = 0;
  }
}
