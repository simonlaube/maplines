let imgDropPaths = null;

function saveEditNote() {
    let comment = document.getElementById('comment-input').value;
    console.log(comment);
}

function dragOverHandler(ev) {
    // console.log('File(s) in drop zone');
    ev.preventDefault();
    document.getElementById("img-drop").style.backgroundColor = "red";
    return false;
}

function dragLeaveHandler(ev) {
    document.getElementById("img-drop").style.backgroundColor = "";
    return false;
}

var imgDropHovered = false;
function imgDropEnter() {
    imgDropHovered = true;
}

function imgDropLeave() {
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
    imgDropHovered = false;
    document.getElementById("img-drop").style.backgroundColor = "";
    imgDropPaths = event.payload; // array of dropped file paths
    if (imgDropPaths[0].endsWith(".JPG")) {
        // document.getElementById("img-drop").style.backgroundImage = "url('" + imgDropPaths[0] + "')";
        // var img = new Image(imgDropPaths[0]);
        console.log("path: " + imgDropPaths[0]);
        let src = convertFileSrc(imgDropPaths[0]);
        console.log("url: " + src);
        document.getElementById("note-img").style.backgroundImage = "url('" + src + "')";


    }
  }
})
