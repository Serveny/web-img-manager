# Web Image Manager

Little project to up and download image files to/from an actix rust server and get live notification with web sockets. It is intended for a web application in which there are different lobbies, each of which has rooms where you can chat or share your pictures with others.

<p align="center">
  <img src="demo_animation.gif"/>
</p>

## Features

- [x] ☁️ upload image
- [x] 🖼️ convert image into big and thumb/preview images
- [x] 📁 save files in structured folder (lobby_id/room_id/img_name)
- [x] ❌ delete lobby & room folders
- [x] ❌ delete images
- [x] 📰 upload live notification
- [x] 📰 delete lobby/room/image live notification
- [x] 💬 live chat
- [ ] 🔗 TS bindins for public messages
- [ ] 👁️ admin control panel (for image deletion & overview web socket connections)

## Examples

- [x] 🧸 Plain HTML/JS example
- [x] 🧸 HTML/TS example frontend application with usable communication class

## Setup with Docker

1. **Create web-image-manager storage volume**: `docker volume create wim-storage`
2. **Build**: `docker build -t web-img-manager .`
3. **Run in interactive mode**: `docker run -p 1871:1871 -v wim-storage:/wim-storage --rm --name wim -it web-img-manager bash` to create and edit config
   - **Copy**: `cp /wim-storage/config/default-server-config.json /wim-storage/config/server-config.json`
   - **Edit**: `nano /wim-storage/config/server-config.json` Change `images_storage_path` to `/wim-storage/pictures`
   - **Exit**: After saving file write exit in console
4. **Run**: `docker run -p 1871:1871 -v wim-storage:/wim-storage --rm --name wim web-img-manager`

## API

#### Types

- `lobby_id`: Uuid v4
- `room_id`: Uuid v4
- `session_id`: Uuid v4
- `img_id`: 32 bit Integer
- `Permission`: Object
  - `url_whitelist`: Restrict access to urls. If None, every url is allowed
  - `restriction`: Rescriction enum
- `Restriction`: Choose one option <ul>
  <li><code>AllowedToAll</code>: Allow access from anywhere by anyone</li>
  <li><code>NeedsConfimation</code>: Allow access after sending serverside confirmation request</li>
  <li><code>Denied</code>: Deny access</li>
</ul>

#### Public requests

<table>
  <tr>
    <th>Function</th>
    <th>Method</th>
    <th>URL</th>
    <th>Parameters</th>
    <th>Return format</th>
    <th>Returns</th>
  </tr>
  <tr>
    <td>get room list for lobby</td>
    <td>GET</td>
    <td><code>/list/{lobby_id}/</code></td>
    <td>None</td>
    <td>JSON</td>
    <td>JSON encoded list of room_id's <br><code>["9b5938c0-ae34-42a4-b459-06124ae70ffa", "10f70fb4-c9c7-4c0d-abcf-13e2cd49a85a"]</code></td>
  </tr>
  <tr>
    <td>get image name list for room</td>
    <td>GET</td>
    <td><code>/list/{lobby_id}/{room_id}</code></td>
    <td>None</td>
    <td>JSON</td>
    <td>JSON encoded list of int img_id's <br><code>[1,2,3,4,8]</code></td>
  </tr>
  <tr>
    <td>get thumb image</td>
    <td>GET</td>
    <td><code>/img/thumb/{lobby_id}/{room_id}/{img_id}</code></td>
    <td>None</td>
    <td>.jpg</td>
    <td>thumb image file</td>
  </tr>
  <tr>
    <td>get big image</td>
    <td>GET</td>
    <td><code>/img/{lobby_id}/{room_id}/{img_id}</code></td>
    <td>None</td>
    <td>.jpg</td>
    <td>image file</td>
  </tr>
  <tr>
    <td>upload</td>
    <td>POST</td>
    <td><code>/upload/{lobby_id}/{room_id}</code></td>
    <td><code>image</code>: Image as form file</td>
    <td>JSON</td>
    <td>image upload result<br><code>{ img_id: 3 }</code></td>
  </tr>
  <tr>
    <td>connect to websocket</td>
    <td>GET</td>
    <td><code>/ws/{lobby_id}</code></td>
    <td>None</td>
    <td>JSON</td>
    <td>null</td>
  </tr>
  <tr>
    <th colspan="6"><br>Admin requests</th>
  </tr>
 <tr>
    <td>delete room / lobby / img</td>
    <td>POST</td>
    <td><code>/delete/{lobby_id?}/{room_id?}/{img_id?}</code></td>
    <td>None</td>
    <td>JSON</td>
    <td>null</td>
  </tr>
<tr>
    <td>send chat message</td>
    <td>POST</td>
    <td><code>/chat</code></td>
    <td><code>lobby_id</code>: String<br><code>msg</code>: String<br></td>
    <td>JSON</td>
    <td>null</td>
  </tr>
</table>

### Web sockets messages

<table>
  <tr>
    <th>Direction</th>
    <th>Function</th>
    <th>Format</th>
    <th>Content</th>
  </tr>
  <tr>
    <td>Server -> Client</td>
    <td>Self connected notification</td>
    <td>JSON</td>
    <td><code>event</code>: "Connected", <code>session_id</code></td>
  </tr>
  <tr>
    <td>Server -> Client</td>
    <td>Image uploaded notification</td>
    <td>JSON</td>
    <td><code>event</code>: "ImageUploaded", <code>room_id</code>, <code>img_id</code></td>
  </tr>
  <tr>
    <td>Server -> Client</td>
    <td>Image deleted notification</td>
    <td>JSON</td>
    <td><code>event</code>: "ImageDeleted", <code>room_id</code>, <code>img_id</code></td>
  </tr>
  <tr>
    <td>Server -> Client</td>
    <td>Chat message notification</td>
    <td>JSON</td>
    <td><code>event</code>: "ChatMessage", <code>username</code>, <code>msg</code></td>
  </tr>
</table>

### Server configuration

<table>
  <tr>
    <th>Property</th>
    <th>Description</th>
    <th>Default Value</th>
  </tr>
  <tr>
    <td><code>url</code></td>
    <td>Server url</td>
    <td><code>0.0.0.0</code></td>
  </tr>
  <tr>
    <td><code>port</code></td>
    <td>Server port</td>
    <td><code>1871</code></td>
  </tr>
  <tr>
    <td><code>images_storage_path</code></td>
    <td>Path for storing all uploaded images</td>
    <td><code>./img-storage</code></td>
  </tr>
  <tr>
    <td><code>max_image_size_byte</code></td>
    <td>maximum input image file size in bytes</td>
    <td><code>20971520</code></td>
  </tr>
  <tr>
    <td><code>upload</code></td>
    <td>Permission for image upload</td>
    <td><code>{ "restriction": "AllowedToAll" }</code></td>
  </tr>
</table>
