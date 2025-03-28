import { Notifications, NotificationsProtocol } from './notifications';
import { Success, UploadResult } from './rs-bindings';

/**
 * @fileOverview Bindings for web img manager
 * @author Serveny
 * @version 1.0.0
 */

export type LobbyId = string;
export type SessionId = string;
export type RoomId = number;
export type ImgId = number;
export type Protocol = 'http' | 'https';

/** Class for communication with web img manager server. */
export class WebImgManager {
  constructor(public server_addr: string, public protocol: Protocol = 'http') {}

  async get_room_img_list(
    lobby_id: LobbyId,
    room_id: RoomId
  ): Promise<ImgId[]> {
    return this.send(
      `${this.protocol}://${this.server_addr}/list/${lobby_id}/${room_id}`,
      'GET'
    );
  }

  img_src(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId): string {
    return `${this.protocol}://${this.server_addr}/img/${lobby_id}/${room_id}/${img_id}`;
  }

  thumb_img_src(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId): string {
    return `${this.protocol}://${this.server_addr}/img/thumb/${lobby_id}/${room_id}/${img_id}`;
  }

  async upload_img(
    lobby_id: LobbyId,
    room_id: RoomId,
    image: File
  ): Promise<UploadResult> {
    const url = `${this.protocol}://${this.server_addr}/upload/${lobby_id}/${room_id}`;
    const formData = new FormData();
    formData.append('image', image);

    const response = await fetch(url, {
      method: 'POST',
      body: formData,
    });

    if (response.ok !== true) throw Error(await response.text());
    return response.json();
  }

  async delete(
    lobby_id: LobbyId,
    room_id?: RoomId,
    img_id?: ImgId
  ): Promise<Success> {
    let url = `${this.protocol}://${this.server_addr}/delete/${lobby_id}`;
    if (room_id != null) url += `/${room_id}`;
    if (room_id != null && img_id != null) url += `/${img_id}`;
    return this.send(url, 'POST');
  }

  async sendChatMessage(lobby_id: LobbyId, msg: string): Promise<Success> {
    let url = `${this.protocol}://${this.server_addr}/chat`;
    return this.send(url, 'POST', { lobby_id, msg });
  }

  connect(
    lobby_id: LobbyId,
    notifications_protocol: NotificationsProtocol = 'ws'
  ): Notifications {
    return new Notifications(
      this.server_addr,
      lobby_id,
      notifications_protocol
    );
  }

  private async send<TRes>(
    url: string,
    method: string,
    params?: object
  ): Promise<TRes> {
    const response = await fetch(url, {
      method: method,
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(params),
    });

    if (response.ok !== true) {
      throw new Error(
        `Response error: ${response.status} - ${
          response.statusText
        } - ${await response.text()}`
      );
    }

    return response.json();
  }
}
