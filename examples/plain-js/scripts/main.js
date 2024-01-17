const server_addr = '127.0.0.1:8080';
const lobby_id = '6a766d31-71d5-4a34-8df5-124b9614b19f';
const room_id = 'dbc64507-38fa-45e4-ad4b-197a4961bfa6';
const web_img_manager = new WebImgManager(server_addr).connect(lobby_id);
const notify = web_img_manager.notifications;

// Subscribe notification events
notify.onConnected((ev) => console.log('WS connected:', ev));
notify.onDisconnected((ev) => console.log('WS disconnected:', ev));
notify.onError((ev) => console.log('WS error:', ev));
notify.onImageUploaded((ev) => addImgs(ev.img_id));
notify.onImageDeleted((ev) => removeImgs(ev.img_id));
notify.onLobbyDeleted((ev) => removeLobby());
notify.onRoomDeleted((ev) => removeLobby());

addRoomImgsToHtml();

async function addRoomImgsToHtml() {
  const img_ids = await web_img_manager.get_room_img_list(lobby_id, room_id);
  for (const img_id of img_ids) addImgs(img_id);
}

function addImgs(imgId) {
  const divs = document.getElementsByTagName('div');
  if (divs[1].querySelectorAll(`img[data-img-id='${imgId}']`).length > 0)
    return;
  addImg(imgId, divs, true);
  addImg(imgId, divs, false);
}

function addImg(imgId, divs, isThumb) {
  const imgEl = document.createElement('img');
  imgEl.src = `http://${server_addr}/img/${
    isThumb ? 'thumb/' : ''
  }${lobby_id}/${room_id}/${imgId}`;
  imgEl.setAttribute('data-img-id', imgId);
  imgEl.style = 'float:left; max-width: 100%';
  if (isThumb) divs[1].append(imgEl);
  else divs[2].append(imgEl);
}

function removeImgs(imgId) {
  document
    .querySelectorAll(`img[data-img-id='${imgId}']`)
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
  const file = await readFile(imageInput.files[0]);
  if (file == null) return;
  const { img_id } = await web_img_manager.upload_img(lobby_id, room_id, file);
  addImgs(img_id);
}

async function deleteFirstImage() {
  const firstImage = document
    .getElementsByTagName('img')[0]
    .getAttribute('data-img-id');
  web_img_manager.delete(lobby_id, room_id, firstImage);
}

function emtpyLobby() {
  const divs = document.getElementsByTagName('div');
  divs[1].replaceChildren();
  divs[2].replaceChildren();
}
