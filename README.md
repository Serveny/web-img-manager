# Web Image Manager

Little project to up and download image files to/from an actix rust server and get live notification with web sockets. It is intended for a web application in which there are different lobbies, each of which has rooms where you can chat or share your pictures with others.

## Features

- [x] ☁️ upload image
- [x] 🖼️ convert image into big and thumb/preview images
- [x] 📁 save files in structured folder (lobby_id/room_id/img_name)
- [x] ❌ delete lobby & room folders
- [x] ❌ delete images
- [ ] 📰 upload live notification
- [ ] 📰 delete live notification
- [ ] 💬 live chat

## API

<table>
  <tr>
    <th>Function</th>
    <th>Method</th>
    <th>URL</th>
    <th>Parameters</th>
    <th>Returns</th>
  </tr>
  <tr>
    <td>get image name list</td>
    <td>GET</td>
    <td><code>/list/{lobby_id}/{room_id}</code></td>
    <td>None</td>
    <td>JSON encoded list of filenames</td>
  </tr>
  <tr>
    <td>get image</td>
    <td>GET</td>
    <td><code>/img/{lobby_id}/{room_id}/{img_filename}</code></td>
    <td>None</td>
    <td>image file</td>
  </tr>
  <tr>
    <td>upload</td>
    <td>POST</td>
    <td><code>/upload</code></td>
    <td><code>lobby_id</code>: String<br><code>room_id</code>: String<br><code>image</code>: Image as base64 encoded string</td>
    <td>image name as string (example: <code>3</code>)</td>
  </tr>
  <tr>
    <td>delete</td>
    <td>POST</td>
    <td><code>/delete/{lobby_id?}/{room_id?}/{img_filename?}</code></td>
    <td>None</td>
    <td>OK</td>
  </tr>
</table>
