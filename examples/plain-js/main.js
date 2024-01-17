const server = '127.0.0.1:8080';
const lobby_id = '6a766d31-71d5-4a34-8df5-124b9614b19f';
const room_id = 'dbc64507-38fa-45e4-ad4b-197a4961bfa6';
const socket = subscribeNotifications();

readImageList();

function readImageList() {
  fetch(`http://${server}/list/${lobby_id}/${room_id}`)
    .then((response) => {
      if (!response.ok)
        throw new Error(
          `Error loading list: ${response.status} - ${response.statusText}`
        );
      return response.json();
    })
    .then((data) => {
      for (const imgId of data) addImgs(imgId);
    });
}

function addImgs(imgId) {
  const divs = document.getElementsByTagName('div');
  addImg(imgId, divs, true);
  addImg(imgId, divs, false);
}

function addImg(imgId, divs, isThumb) {
  const imgEl = document.createElement('img');
  imgEl.src = `http://${server}/img/${
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

  if (file == null) {
    alert('Please select an image.');
    return;
  }

  fetch(`http://${server}/upload`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      lobby_id: lobby_id,
      room_id: room_id,
      image: file,
    }),
  })
    .then((response) => {
      if (!response.ok) {
        throw new Error(
          `Error uploading image: ${response.status} - ${response.statusText}`
        );
      }
      console.log('Upload successfull:', response.body);
    })
    .catch((error) => console.error('Error:', error));
}

function subscribeNotifications() {
  const socket = new WebSocket(`ws://${server}/ws/${lobby_id}`);
  const sessionId = null;

  socket.addEventListener('open', (event) => {
    console.log('Web socket connection opened:', event);
  });

  socket.addEventListener('error', (event) => {
    console.error('Error connecting web socket:', event);
  });

  socket.addEventListener('close', (event) => {
    console.log('web socket connection closed:', event);
  });

  socket.addEventListener('message', (event) => {
    const ev = JSON.parse(event.data);
    console.log('Event:', ev);
    switch (ev.event) {
      case 'Connected':
        sessionId = ev.session_id;
        break;
      case 'ImageUploaded':
        addImgs(ev.img_id);
        break;
      case 'ImageDeleted':
        removeImgs(ev.img_id);
        break;
    }
  });

  return socket;
}

function deleteOnServer(lobbyId, roomId, imgName) {
  let url = `http://${server}/delete/${lobbyId}`;
  if (roomId) url += `/${roomId}`;
  if (imgName) url += `/${imgName}`;
  fetch(url, {
    method: 'POST',
  }).catch((err) => console.error(err));
}
