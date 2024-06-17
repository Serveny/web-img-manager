# Web-Img-Manager Bindings

This is an npm package that provides a TypeScript class to interact with the Rust-based web-img-manager server (https://github.com/Serveny/web-img-manager). It allows for uploading and downloading image files, and receiving live notifications via WebSockets. This package is designed for use in a web application where different lobbies for chatting and contain rooms for sharing pictures with others.

## Features

- Upload images to the Web Image Manager server
- Download images from the server
- Receive live notifications using WebSockets
- Easy integration with web applications

## Installation

To install the package, use npm:

```sh
npm install web-img-manager
```

## Usage

```typescript
import { ImgId, Notifications, RoomId, WebImgManager } from 'web-img-manager';

const server_addr = '127.0.0.1:1871';
const lobby_id = '6a766d31-71d5-4a34-8df5-124b9614b19f';
const web_img_manager = new WebImgManager(server_addr).connect(lobby_id);

// Events
document
  .getElementById('deleteLobbyBtn')
  ?.addEventListener('click', web_img_manager.delete(lobby_id));

// Notifications
const notify = web_img_manager.notifications;
notify?.onConnected((ev) => console.log('WS connected:', ev));
notify?.onDisconnected((ev) => console.log('WS disconnected:', ev));
notify?.onError((ev) => console.log('WS error:', ev));
notify?.onImageUploaded((ev) => addImgs(ev.room_id, ev.img_id));
notify?.onImageDeleted((ev) => removeImgs(ev.room_id, ev.img_id));
notify?.onLobbyDeleted((_) => emtpyLobby());
notify?.onRoomDeleted((ev) => emtpyRoom(ev.room_id));
notify?.onChatMessage((ev) => showChatMessage(ev.username, ev.msg));
```

## License

This project is licensed under the MIT License.
