import { EventEmitter } from './event-emitter';
import {
  ChatMessageEvent,
  ImageProcessedEvent,
  LobbyDeletedEvent,
  RoomDeletedEvent,
  SystemNotificationEvent,
} from './rs-bindings';
import { LobbyId } from './web-img-manager';

export type NotificationsProtocol = 'ws' | 'wss';

/** Class for communication with web img manager web socket server. */
export class Notifications {
  emitter = new EventEmitter();

  constructor(
    server_addr: string,
    lobby_id: LobbyId,
    protocol: NotificationsProtocol = 'ws'
  ) {
    const url = `${protocol}://${server_addr}/notifications/${lobby_id}`;
    const socket = new WebSocket(url);

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

  onConnected(handler: (ev: Event) => void): this {
    this.emitter.on('Connected', handler);
    return this;
  }

  onDisconnected(handler: (ev: Event) => void): this {
    this.emitter.on('Disonnected', handler);
    return this;
  }

  onError(handler: (ev: Event) => void): this {
    this.emitter.on('Error', handler);
    return this;
  }

  onImageUploaded(handler: (ev: ImageProcessedEvent) => void): this {
    this.emitter.on('ImageUploaded', handler);
    return this;
  }

  onLobbyDeleted(handler: (ev: LobbyDeletedEvent) => void): this {
    this.emitter.on('LobbyDeleted', handler);
    return this;
  }

  onRoomDeleted(handler: (ev: RoomDeletedEvent) => void): this {
    this.emitter.on('RoomDeleted', handler);
    return this;
  }

  onImageDeleted(handler: (ev: ImageProcessedEvent) => void): this {
    this.emitter.on('ImageDeleted', handler);
    return this;
  }

  onChatMessage(handler: (ev: ChatMessageEvent) => void): this {
    this.emitter.on('ChatMessage', handler as any);
    return this;
  }

  onSystemNotification(handler: (ev: SystemNotificationEvent) => void): this {
    this.emitter.on('SystemNotification', handler);
    return this;
  }
}
