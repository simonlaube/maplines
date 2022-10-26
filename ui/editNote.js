let imgDropPaths = null;
const pickerOpts = {
    types: [
        {
            description: 'Images',
            accept: {
                'image/*': ['.png', '.gif', '.jpeg', '.jpg']
            }
        },
    ],
    excludeAcceptAllOption: true,
    multiple: false
};
  
function addEditNote() {
    setNoOverlay();
    let comment = document.getElementById('comment-input').value;
    let coords = movableMarker.features[0].geometry.coordinates;
    let trackIcon = document.getElementById("track-icon-input").value;

    console.log(comment);
    console.log(coords);
    console.log(trackIcon);
    console.log(imgPreviewPaths);
    // TODO: Clear all input fields (comment, pitcures, ...)
    invoke('add_note', { ulid: selected_rows[0], coords: coords, icon: trackIcon, comment: comment, img_paths: imgPreviewPaths});
}

async function openFileDialog() {
    var fileSelector = document.createElement('input');
    fileSelector.setAttribute('type', 'file');
    fileSelector.onchange = function(event) {
        console.log(event.target.files);
    }
    fileSelector.click();
    // delete fileSelector;
}

function imgDragEnter() {
    let textField = document.getElementById("img-drop");
    textField.style.color = "#bbb";
    textField.innerHTML = "Drop Picture(s)";
    textField.style.borderColor = "#555";
    imgDropHovered = true;
}

function imgDragEnd() {
    let textField = document.getElementById("img-drop");
    textField.style.color = "";
    textField.innerHTML = "Add Picture(s)";
    textField.style.borderColor = "";
    imgDropHovered = false;
}

var imgPreviewPaths = [];
listen('tauri://file-drop', event => {
  if (imgDropHovered) {
    imgDragEnd();
    imgDropPaths = event.payload; // array of dropped file paths
    let grid = document.getElementById("img-preview-wrapper");
    for (p of imgDropPaths) {
        let ending = p.toLowerCase().split('.');
        ending = ending[ending.length - 1];
        if (['png', 'jpg', 'jpeg', 'gif'].includes(ending)) {
            imgPreviewPaths.push(p);
            let src = convertFileSrc(p);
            console.log("url: " + p);
            let img = document.createElement('img');
            img.style.objectFit = "cover";
            img.style.width = "100%";
            img.style.height = "100%";
            img.style.overflow = "hidden";
            img.style.borderRadius = "5px";
            img.style.boxSizing = "border-box";
            img.classList.add('img-preview');
            img.src = src;

            let text = document.createElement('div');
            text.innerHTML = "Remove Picture";

            text.style.color = "#bc0000";
            text.style.display = "flex";
            text.style.alignItems = 'center';
            text.style.position = "absolute";
            text.style.top = "50%";
            text.style.left = "50%";
            text.style.transform = "translate(-50%, -50%)";
            text.style.fontFamily = "'Vela-Sans'";
            text.style.whiteSpace = "nowrap";
            text.style.fontWeight = "900";
            text.style.pointerEvents = "none";
            text.style.display = "none";
            text.style.backgroundColor = "rgba(255, 255, 255, 0.5)";
            // text.style.border = "4px solid #bc0000";
            text.style.borderRadius = "5px";
            text.style.width = "100%";
            text.style.height = "100%";
            text.style.boxSizing = "border-box";
            text.style.alignItems = "center";
            text.style.justifyContent = "center";

            let imgWrap = document.createElement('div');
            imgWrap.style.width = "100%";
            imgWrap.style.height = "100%";
            imgWrap.style.position = "relative";
            imgWrap.appendChild(img);
            imgWrap.appendChild(text);
            grid.appendChild(imgWrap);
            grid.scrollTop = grid.scrollHeight;

            img.addEventListener('mouseenter', (event) => {
                text.style.display = "flex";
            });
            img.addEventListener('mouseleave', (event) => {
                text.style.display = "none";
            });
            img.addEventListener('click', (response) => {
                // remove path from list
                for (let i = 0; i < imgPreviewPaths.length; i++) {
                    if (p === imgPreviewPaths[i]) {
                        imgPreviewPaths.splice(i, 1);
                        break;
                    }
                }
                imgWrap.remove();
            });
        }
        console.log(imgPreviewPaths);
        // .style.backgroundImage = "url('" + src + "')";
    }
    /*
    if (imgDropPaths[0].endsWith(".JPG")) {
        // document.getElementById("img-drop").style.backgroundImage = "url('" + imgDropPaths[0] + "')";
        // var img = new Image(imgDropPaths[0]);
        console.log("path: " + imgDropPaths[0]);
        let src = convertFileSrc(imgDropPaths[0]);
        console.log("url: " + src);
        document.getElementById("note-img").style.backgroundImage = "url('" + src + "')";


    }
    */
  }
})

function checkNoteInput(field) {
    /*let inputField;
    let button = document.getElementById('save-note-edit');
    if (field === 'icon') {
        inputField = document.getElementById("track-icon-input");
        if (inputField.value === activityDefault) {
            inputField.style.color = "";
        } else {
            inputField.style.color = "#000";
        }
    } /*else if (field === 'name') {
        inputField = document.getElementById("name-input");

        if (!inputIsValid(inputField.value)) {
            document.getElementById("invalid-characters-alert").style.display = "block";
            inputField.style.borderColor = "#ff6473";
            button.style.opacity = "0.6";
            button.style.pointerEvents = "none";
            return;
        }

        if (inputField.value === "" || inputField.value === nameDefault) {
            inputField.value = nameDefault;
            inputField.style.color = "";
        }

        document.getElementById("invalid-characters-alert").style.display = "none";
        inputField.style.borderColor = "";
        if (allInputsValid()) {
            button.style.opacity = "1.0";
            button.style.pointerEvents = "";
        }
    }*/
}

function addNote() {
    if (selected_rows.length != 1) {
        // TODO: Display Dialog "You must select one track."
        return;
    }
    setAddNoteOverlay();
}