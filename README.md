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
    <th>Name</th>
    <th>Method</th>
    <th>URL</th>
    <th>Parameters</th>
    <th>Returns</th>
  </tr>
  <tr>
    <td><code>get image name list</code></td>
    <td>GET</td>
    <td>/list/{lobby_id}/{room_id}</td>
    <td>None</td>
    <td>JSON encoded list of filenames</td>
  </tr>
  <tr>
    <td><code>get image</code></td>
    <td>GET</td>
    <td>/img/{lobby_id}/{room_id}/{img_filename}</td>
    <td>None</td>
    <td>image file</td>
  </tr>
  <tr>
    <td><code>upload</code></td>
    <td>POST</td>
    <td>/upload</td>
    <td><code>lobby_id</code>: String<br><code>room_id</code>: String<br><code>image</code>: Image as base64 encoded string</td>
    <td>image name as string (example: <code>3</code>)</td>
  </tr>
  <tr>
    <td><code>delete</code></td>
    <td>POST</td>
    <td>/delete/{lobby_id?}/{room_id?}/{img_filename?}</td>
    <td>None</td>
    <td>OK</td>
  </tr>
</table>
