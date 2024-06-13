const server_addr = '127.0.0.1:8080';
const lobby_id = '6a766d31-71d5-4a34-8df5-124b9614b19f';
const web_img_manager = new WebImgManager(server_addr).connect(lobby_id);
const notify = web_img_manager.notifications;

// HTML elements
const roomSelect = document.getElementById('room-select');
const lobbyEl = document.getElementById('lobby');
const chatEl = document.getElementById('chat-messages');
const chatInputEl = document.getElementById('chat-input');

// Subscribe notification events
notify.onConnected((ev) => console.log('WS connected:', ev));
notify.onDisconnected((ev) => console.log('WS disconnected:', ev));
notify.onError((ev) => console.log('WS error:', ev));
notify.onImageUploaded((ev) => addImgs(ev.room_id, ev.img_id));
notify.onImageDeleted((ev) => removeImgs(ev.room_id, ev.img_id));
notify.onLobbyDeleted((ev) => emtpyLobby());
notify.onRoomDeleted((ev) => emtpyRoom(ev.room_id));
notify.onChatMessage((ev) => showChatMessage(ev.username, ev.msg));

for (const roomId of [...roomSelect.options].map((o) => o.value))
  addRoomImgsToHtml(roomId);

async function addRoomImgsToHtml(room_id) {
  const img_ids = await web_img_manager.get_room_img_list(lobby_id, room_id);
  for (const img_id of img_ids) addImgs(room_id, img_id);
}

function getOrInsertRoomEl(room_id) {
  let roomEl = lobbyEl.querySelector(`div[data-room-id='${room_id}']`);
  if (!roomEl) {
    roomEl = document.createElement('div');
    roomEl.className = 'room';
    roomEl.setAttribute('data-room-id', room_id);
    roomEl.id = room_id;

    label = document.createElement('label');
    label.htmlFor = room_id;
    label.textContent = document.querySelector(
      `#room-select option[value="${room_id}"`
    ).textContent;

    lobbyEl.appendChild(label);
    lobbyEl.appendChild(roomEl);
  }
  return roomEl;
}

function addImgs(room_id, img_id) {
  const roomEl = getOrInsertRoomEl(room_id);
  if (roomEl.querySelectorAll(`img[data-img-id='${img_id}']`).length > 0)
    return;

  // Add thumb image
  addImg(room_id, img_id, roomEl, true);

  // Add big image
  //addImg(room_id, img_id, roomEl, false);
}

function addImg(room_id, img_id, roomEl, isThumb) {
  const imgEl = document.createElement('img');
  imgEl.src = `http://${server_addr}/img/${
    isThumb ? 'thumb/' : ''
  }${lobby_id}/${room_id}/${img_id}`;
  imgEl.setAttribute('data-img-id', img_id);
  imgEl.style = 'max-width: 100%';
  roomEl.append(imgEl);
  roomEl.append(imgEl);
}

function removeImgs(room_id, img_id) {
  getRoomElById(room_id)
    .querySelectorAll(`img[data-img-id='${img_id}']`)
    .forEach((imgEl) => imgEl.parentNode.removeChild(imgEl));
}

async function readFile(file) {
  return new Promise((resolve) => {
    const reader = new FileReader();
    reader.onload = (ev) => resolve(ev.target?.result);
    reader.readAsDataURL(file);
  });
}

async function uploadImage() {
  const imageInput = document.getElementById('imageInput');
  const room_id = roomSelect.value;
  for (const file of imageInput.files) {
    if (file == null) continue;
    web_img_manager.upload_img(lobby_id, room_id, file).then(({ img_id }) => {
      if (web_img_manager.notifications == null) addImgs(room_id, img_id);
    });
  }
}

async function deleteFirstImage() {
  const roomId = roomSelect.value;
  const firstImage = getRoomElById(roomId)
    .getElementsByTagName('img')[0]
    .getAttribute('data-img-id');
  web_img_manager.delete(lobby_id, roomId, firstImage);
}

function getRoomElById(room_id) {
  return document.querySelector(`div[data-room-id='${room_id}']`);
}

function emtpyRoom(room_id) {
  getRoomElById(room_id).replaceChildren();
}

function emtpyLobby() {
  lobbyEl.replaceChildren();
}

function sendChatMessage() {
  const chatInputEl = document.getElementById('chat-input');
  web_img_manager.sendChatMessage(lobby_id, chatInputEl.value);
  chatInputEl.value = '';
}

function showChatMessage(username, msg) {
  const msgEl = document.createElement('div');
  msgEl.innerHTML = `<b>${username}:</b> ${msg}`;
  chatEl.append(msgEl);
  chatEl.scrollTop = chatEl.scrollHeight - chatEl.clientHeight;
}
