import { Notifications } from './notifications';
import { Success, UploadResult } from './rs-bindings';

/**
 * @fileOverview Bindings for web img manager
 * @author Serveny
 * @version 1.0.0
 */

export type LobbyId = string;
export type RoomId = number;
export type ImgId = number;

/** Class for communication with web img manager server. */
export class WebImgManager {
  notifications: Notifications | null = null;

  constructor(public server_addr: string) {}

  async get_room_img_list(
    lobby_id: LobbyId,
    room_id: RoomId
  ): Promise<ImgId[]> {
    return send(
      `http://${this.server_addr}/list/${lobby_id}/${room_id}`,
      'GET'
    );
  }

  img_src(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId): string {
    return `http://${this.server_addr}/img/${lobby_id}/${room_id}/${img_id}`;
  }

  thumb_img_src(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId): string {
    return `http://${this.server_addr}/img/thumb/${lobby_id}/${room_id}/${img_id}`;
  }

  async upload_img(
    lobby_id: LobbyId,
    room_id: RoomId,
    image: File
  ): Promise<UploadResult> {
    const url = `http://${this.server_addr}/upload/${lobby_id}/${room_id}`;
    const formData = new FormData();
    formData.append('image', image);

    const res = await fetch(url, {
      method: 'POST',
      body: formData,
    });

    return res.json();
  }

  async delete(
    lobby_id: LobbyId,
    room_id?: RoomId,
    img_id?: ImgId
  ): Promise<Success> {
    let url = `http://${this.server_addr}/delete/${lobby_id}`;
    if (room_id != null) url += `/${room_id}`;
    if (room_id != null && img_id != null) url += `/${img_id}`;
    return send(url, 'POST');
  }

  async sendChatMessage(lobby_id: LobbyId, msg: string): Promise<Success> {
    let url = `http://${this.server_addr}/chat`;
    return send(url, 'POST', { lobby_id, msg });
  }

  connect(lobby_id: LobbyId): WebImgManager {
    this.notifications = new Notifications(this.server_addr, lobby_id);
    return this;
  }
}

async function send<TRes>(
  url: string,
  method: string,
  params?: object
): Promise<TRes> {
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
