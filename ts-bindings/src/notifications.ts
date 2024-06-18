import { EventEmitter } from './event-emitter';
import {
  ChatMessageEvent,
  ImageProcessedEvent,
  LobbyDeletedEvent,
  RoomDeletedEvent,
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
    const socket = new WebSocket(`${protocol}://${server_addr}/ws/${lobby_id}`);

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

  onImageUploaded(handler: (ev: ImageProcessedEvent) => void) {
    this.emitter.on('ImageUploaded', handler as any);
  }

  onLobbyDeleted(handler: (ev: LobbyDeletedEvent) => void) {
    this.emitter.on('LobbyDeleted', handler as any);
  }

  onRoomDeleted(handler: (ev: RoomDeletedEvent) => void) {
    this.emitter.on('RoomDeleted', handler as any);
  }

  onImageDeleted(handler: (ev: ImageProcessedEvent) => void) {
    this.emitter.on('ImageDeleted', handler as any);
  }

  onChatMessage(handler: (ev: ChatMessageEvent) => void) {
    this.emitter.on('ChatMessage', handler as any);
  }
}
