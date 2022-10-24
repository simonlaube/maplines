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
  
function saveEditNote() {
    let comment = document.getElementById('comment-input').value;
    console.log(comment);
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
    // textField.style.color = "#f00";
    imgDropHovered = true;
}

function imgDragEnd() {
    let textField = document.getElementById("img-drop");
    textField.style.color = "";
    textField.innerHTML = "Add Picture(s)";
    textField.style.borderColor = "";
    imgDropHovered = false;
}

function initDropListener() {
    /*let input = document.getElementById('img-drop');
    input.addEventListener("change", (e) => {
        let file = input.files[0];
        console.log(input.files[0])
        let url = URL.createObjectURL(file);
        console.log("url: " + url);
        document.getElementById("img-drop").style.backgroundImage = "url('" + url + "')";
    });*/
}

listen('tauri://file-drop', event => {
  if (imgDropHovered) {
    // console.log(event)
    imgDragEnd();
    // document.getElementById("img-drop").style.backgroundColor = "";
    imgDropPaths = event.payload; // array of dropped file paths
    let grid = document.getElementById("img-preview-wrapper");
    for (p of imgDropPaths) {
        let src = convertFileSrc(p);
        console.log("url: " + src);
        let img = document.createElement('img');
        img.style.width = "100%";
        img.style.height = "50%";
        // img.style.display = "block";
        img.style.objectFit = "cover";
        img.style.overflow = "hidden";
        img.style.borderRadius = "5px";
        img.src = src;
        grid.appendChild(img);
        grid.scrollTop = grid.scrollHeight;
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
