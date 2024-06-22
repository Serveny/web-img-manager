import { ImgId, Notifications, RoomId, WebImgManager } from 'web-img-manager';

const server_addr = '127.0.0.1:1871';
const lobby_id = '6a766d31-71d5-4a34-8df5-124b9614b19f';
const web_img_manager = new WebImgManager(server_addr, 'http');
web_img_manager.connect(lobby_id, 'ws');
const notify = web_img_manager.notifications;
if (notify) addNotifications(notify);
else console.warn('Notifications not available');
addButtonHandler();

// HTML elements
const roomSelect = document.getElementById('room-select')! as HTMLSelectElement;
const lobbyEl = document.getElementById('lobby')!;
const chatEl = document.getElementById('chat-messages')!;

// Subscribe notification events
function addNotifications(notify: Notifications) {
  notify.onConnected((ev) => console.log('WS connected:', ev));
  notify.onDisconnected((ev) => console.log('WS disconnected:', ev));
  notify.onError((ev) => console.log('WS error:', ev));
  notify.onImageUploaded((ev) => addImgs(ev.room_id, ev.img_id));
  notify.onImageDeleted((ev) => removeImgs(ev.room_id, ev.img_id));
  notify.onLobbyDeleted((_) => emtpyLobby());
  notify.onRoomDeleted((ev) => emtpyRoom(ev.room_id));
  notify.onChatMessage((ev) => showChatMessage(ev.username, ev.msg));
}

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
  const img_ids = await web_img_manager.get_room_img_list(lobby_id, room_id);
  for (const img_id of img_ids) addImgs(room_id, img_id);
}

function getOrInsertRoomEl(room_id: RoomId) {
  let roomEl = lobbyEl.querySelector(`div[data-room-id='${room_id}']`);
  if (!roomEl) {
    roomEl = document.createElement('div');
    roomEl.className = 'room';
    const roomId = room_id.toString();
    roomEl.setAttribute('data-room-id', roomId);
    roomEl.id = roomId;

    const label = document.createElement('label');
    label.htmlFor = roomId;
    label.textContent =
      document.querySelector(`#room-select option[value="${roomId}"`)
        ?.textContent ?? '';

    lobbyEl.appendChild(label);
    lobbyEl.appendChild(roomEl);
  }
  return roomEl;
}

function addImgs(room_id: RoomId, img_id: ImgId) {
  const roomEl = getOrInsertRoomEl(room_id);
  if (roomEl.querySelectorAll(`img[data-img-id='${img_id}']`).length > 0)
    return;

  // Add thumb image
  addImg(room_id, img_id, roomEl as HTMLElement, true);

  // Add big image
  //addImg(room_id, img_id, roomEl, false);
}

function addImg(
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
  roomEl.append(imgEl);
  roomEl.append(imgEl);
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
        if (web_img_manager.notifications == null) addImgs(room_id, img_id);
      })
      .catch((err) => console.error(err));
  }
}

async function deleteFirstImage() {
  const roomId = parseInt(roomSelect.value);
  const firstImage =
    getRoomElById(roomId)
      ?.getElementsByTagName('img')[0]
      .getAttribute('data-img-id') ?? '0';
  if (firstImage)
    web_img_manager.delete(lobby_id, roomId, parseInt(firstImage));
}

function getRoomElById(room_id: RoomId) {
  return document.querySelector(`div[data-room-id='${room_id}']`);
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
