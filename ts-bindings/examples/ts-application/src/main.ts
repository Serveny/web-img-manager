import { ImgId, RoomId, WebImgManager } from 'web-img-manager';

const server_addr = '127.0.0.1:1871';
const lobby_id = '6a766d31-71d5-4a34-8df5-124b9614b19f';
const web_img_manager = new WebImgManager(server_addr, 'http');
const notifications = web_img_manager.connect(lobby_id, 'ws');

// Subscribe notification events
notifications
  ?.onConnected((ev) => console.log('WS connected:', ev))
  .onDisconnected((ev) => console.log('WS disconnected:', ev))
  .onError((ev) => showError(ev))
  .onImageUploaded((ev) => prependImgWithRoom(ev.room_id, ev.img_id))
  .onImageDeleted((ev) => removeImgs(ev.room_id, ev.img_id))
  .onLobbyDeleted((_) => emtpyLobby())
  .onRoomDeleted((ev) => emtpyRoom(ev.room_id))
  .onChatMessage((ev) => showChatMessage(ev.username, ev.msg))
  .onSystemNotification((ev) => console.info(ev.msg_type, ev.msg)) ??
  console.warn('Notifications not available');
addButtonHandler();

// HTML elements
const roomSelect = document.getElementById('room-select')! as HTMLSelectElement;
const lobbyEl = document.getElementById('lobby')!;
const chatEl = document.getElementById('chat-messages')!;

function addButtonHandler() {
  const add = (id: string, handler: () => void) =>
    document.getElementById(id)?.addEventListener('click', handler);
  add('uploadBtn', () => uploadImage());
  add('deleteLobbyBtn', () => web_img_manager.delete(lobby_id));
  add('deleteRoomBtn', () =>
    web_img_manager.delete(lobby_id, parseInt(roomSelect.value))
  );
  add('deleteFirstImgBtn', () => deleteFirstImage());
  add('sendChatMsgBtn', () => sendChatMessage());
  document.getElementById('chat-input')?.addEventListener('keypress', (ev) => {
    if (ev.key === 'Enter') sendChatMessage();
  });
}

for (const roomId of [...roomSelect.options].map((o) => o.value))
  addRoomImgsToHtml(parseInt(roomId));

async function addRoomImgsToHtml(room_id: RoomId) {
  (await web_img_manager.get_room_img_list(lobby_id, room_id))
    .reverse()
    .forEach((img_id) => prependImgWithRoom(room_id, img_id));
}

function getOrInsertRoomEl(room_id: RoomId): HTMLDivElement {
  let roomConEl = lobbyEl.querySelector(`details[data-room-id='${room_id}']`);
  if (!roomConEl) {
    const roomImgsEl = document.createElement('p');
    const roomId = room_id.toString();
    roomImgsEl.setAttribute('data-room-id', roomId);
    roomImgsEl.className = 'room-imgs';
    const label = document.createElement('summary');
    label.textContent =
      document.querySelector(`#room-select option[value="${roomId}"`)
        ?.textContent ?? '';

    roomConEl = document.createElement('details');
    roomConEl.classList.add('room');
    roomConEl.appendChild(label);
    roomConEl.appendChild(roomImgsEl);
    roomConEl.setAttribute('data-room-id', roomId);
    roomConEl.setAttribute('style', `order:${roomId}`);
    roomConEl.setAttribute('open', '');

    lobbyEl.appendChild(roomConEl);
  }
  return roomConEl as HTMLDivElement;
}

function prependImgWithRoom(room_id: RoomId, img_id: ImgId) {
  const roomEl = getOrInsertRoomEl(room_id);
  if (roomEl.querySelectorAll(`img[data-img-id='${img_id}']`).length > 0)
    return;

  // Add thumb image
  const roomImgsEl = roomEl.querySelector('.room-imgs');
  if (!roomImgsEl) return;
  prependImg(room_id, img_id, roomImgsEl as HTMLElement, true);

  // Add big image
  //addImg(room_id, img_id, roomEl, false);
}

function prependImg(
  room_id: RoomId,
  img_id: ImgId,
  roomEl: HTMLElement,
  isThumb: boolean
) {
  const imgEl = document.createElement('img');
  imgEl.src = isThumb
    ? web_img_manager.thumb_img_src(lobby_id, room_id, img_id)
    : web_img_manager.img_src(lobby_id, room_id, img_id);
  imgEl.setAttribute('data-img-id', img_id.toString());
  imgEl.setAttribute('style', 'max-width: 100%');
  roomEl.prepend(imgEl);
}

function removeImgs(room_id: RoomId, img_id: ImgId) {
  getRoomElById(room_id)
    ?.querySelectorAll(`img[data-img-id='${img_id}']`)
    .forEach((imgEl) => imgEl.parentNode?.removeChild(imgEl));
}

async function uploadImage() {
  const imageInput = document.getElementById('imageInput')! as HTMLInputElement;
  const room_id = parseInt(roomSelect.value);
  for (const file of imageInput.files!) {
    if (file == null) continue;
    web_img_manager
      .upload_img(lobby_id, room_id, file)
      .then(({ img_id }) => {
        if (notifications == null) prependImgWithRoom(room_id, img_id);
      })
      .catch((err) => showError(err));
  }
}

async function deleteFirstImage() {
  const roomId = parseInt(roomSelect.value);
  const firstImage = getRoomElById(roomId)
    ?.getElementsByTagName('img')?.[0]
    .getAttribute('data-img-id');
  if (firstImage)
    web_img_manager.delete(lobby_id, roomId, parseInt(firstImage));
  else console.info('No image found');
}

function getRoomElById(room_id: RoomId) {
  return document.querySelector(`details[data-room-id='${room_id}']`);
}

function emtpyRoom(room_id: RoomId) {
  getRoomElById(room_id)?.replaceChildren();
}

function emtpyLobby() {
  lobbyEl.replaceChildren();
}

function sendChatMessage() {
  const chatInputEl = document.getElementById(
    'chat-input'
  )! as HTMLInputElement;
  web_img_manager.sendChatMessage(lobby_id, chatInputEl.value);
  chatInputEl.value = '';
}

function showChatMessage(username: string, msg: string) {
  const msgEl = document.createElement('div');
  msgEl.innerHTML = `<b>${username}:</b> ${msg}`;
  chatEl.append(msgEl);
  chatEl.scrollTop = chatEl.scrollHeight - chatEl.clientHeight;
}

let errorBoxTimeout: number | null = null;

function showError(err: Error | any) {
  console.error(err);
  const errorBox = document.getElementById('error-box') as HTMLDivElement;
  if (errorBoxTimeout != null) clearTimeout(errorBoxTimeout);
  errorBox.textContent = err instanceof Error ? err.message : err.toString();
  errorBox.style.display = 'block';

  errorBoxTimeout = setTimeout(() => {
    errorBox.style.display = 'none';
    errorBox.textContent = '';
    errorBoxTimeout = null;
  }, 6000);
}
