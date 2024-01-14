# Web Image Manager

Little project to up and download image files to/from an actix rust server and get live notification with web sockets.

## Features

- [x] upload image
- [x] convert image into big and thumb/preview images
- [x] save files in structured folder (room_id/chapter_id/img_name)
- [x] delete room & chapter folders
- [x] delete images
- [ ] upload live notification
- [ ] delete live notification
- [ ] live chat

## API

### 1. **get image name list**

**Endpoint**

- Method: `GET`
- URL: `/list/{room_id}/{chapter_id}`

**Parameters:** None  
**Returns:** JSON encoded list of filenames (example: `["1.jpg","1_thumb.jpg","2.jpg","2_thumb.jpg"]`)

### 2. **get image**

**Endpoint**

- Method: `GET`
- URL: `/img/{room_id}/{chapter_id}/{img_filename}`

**Parameters:**: None  
**Returns:** image file

### 3. `upload`

**Endpoint**

- **Method:** `POST`
- **URL:** `/upload`

**Parameters**

- `room_id`: String
- `chapter_id`: String
- `image`: Image as base64 encoded string

**Returns:** image name as string (example: `3`)

### 3. `delete`

**Endpoint**

- **Method:** `POST`
- **URL:** `/delete/{room_id?}/{chapter_id?}/{img_filename?}`

**Parameters:** None  
**Returns:** OK or ERROR
