// access the pre-bundled global API functions
const invoke = window.__TAURI__.invoke;
const { emit, listen } = window.__TAURI__.event;
const { convertFileSrc } = window.__TAURI__.tauri;
/*
listen('tauri://file-drop', event => {
  console.log(event)
})*/

var map;
var mapPositionIcon;
var table_body;
var row_objects;
var selected_rows;
var elevationCoords = {};

const DisplayState = {
    Table: 'Table',
    Analysis: 'Analysis',
    Map: 'Map',
}
var currentDisplayState = DisplayState.Analysis;

const OverlayState = {
    None: 'None',
    RowEdit: 'RowEdit',
    NoteEdit: 'NoteEdit',
    Loading: 'Loading',
    Settings: 'Settings',
    AddNote: 'AddNote',
    DisplayNote: 'DisplayNote',
};
var currentOverlayState = OverlayState.None;

window.onload = init;

function init() {
    initContentResize();
    reloadTable();
    initMap();
    initMapPositionIcon();
    selected_rows = [];

    let bTableMap = document.getElementById("table-map");
    bTableMap.style.opacity = "0.6";
    bTableMap.style.pointerEvents = "none";
}

listen("track_import", ev => {
    add_to_table(ev.payload, true);
});

function initContentResize() {
    let vSeparator = document.getElementById("ver-content-separator");
    vSeparator.addEventListener("mousedown", (e) => {
        /*e.preventDefault();
        let percentLeft = e.clientX / e.view.innerWidth * 100;
        console.log(percentLeft);
        content.style.gridAutoColumns = percentLeft + "% 0.15rem auto";*/
        startVDrag(e);
    });

    let hSeparator = document.getElementById("hor-content-separator");
    hSeparator.addEventListener("mousedown", (e) => {
            startHDrag(e);
            // document.getElementById('')
    });
}

function disableSelect(event) {
    event.preventDefault();
}

function startHDrag(event) {
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "ns-resize";
    document.getElementById('content-wrapper').style.cursor = "ns-resize";
    document.getElementById('elevation-graph').style.cursor = "ns-resize";
    document.getElementById('hor-content-separator').style.cursor = "ns-resize";
    document.getElementById('content-wrapper').style.pointerEvents = "none";
    window.addEventListener('mouseup', onHDragEnd);
    window.addEventListener('selectstart', disableSelect);
    window.addEventListener('mousemove', moveHSeparator);
    
}

function onHDragEnd() {
    window.removeEventListener('mouseup', onHDragEnd);
    window.removeEventListener('selectstart', disableSelect);
    window.removeEventListener('mousemove', moveHSeparator);
    document.getElementById('content-wrapper').style.cursor = "";
    document.getElementById('elevation-graph').style.cursor = "";
    document.getElementById('content-wrapper').style.pointerEvents = "";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "";
    elevationGraph.resize();
}

function moveHSeparator(e) {
    // let percentTop = e.clientY / e.view.innerHeight * 100;
    let percentTop = e.clientY / e.view.innerHeight * 100;
    let content = document.getElementById("content-wrapper");
    content.style.gridAutoRows = percentTop + "% 0.15rem auto";
    elevationGraph.resize();
}

function startVDrag(event) {
    document.getElementById('content-wrapper').style.cursor = "ew-resize";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "ew-resize";
    window.addEventListener('mouseup', onVDragEnd);
    window.addEventListener('selectstart', disableSelect);
    window.addEventListener('mousemove', moveVSeparator);
    
}

function onVDragEnd() {
    window.removeEventListener('mouseup', onVDragEnd);
    window.removeEventListener('selectstart', disableSelect);
    window.removeEventListener('mousemove', moveVSeparator);
    document.getElementById('content-wrapper').style.cursor = "";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "";
    map.resize();
    elevationGraph.resize();
}

function moveVSeparator(e) {
    let percentLeft = e.clientX / e.view.innerWidth * 100;
    let content = document.getElementById("content-wrapper");
    content.style.gridAutoColumns = percentLeft + "% 0.15rem auto";
    map.resize();
    elevationGraph.resize();
}

function setCursorStyle() {
    document.getElementById("content-wrapper").style.cursor = "wait";
}

function list_gpx_files() {
    invoke('list_gpx_files')
        .then((response) => console.log(response));
}

function reloadTable() {
    table_body = document.getElementById("gpxTableBody");
    row_objects = {};
    invoke('load_track_analysis')
        .then((response) => {
            while(table_body.rows.length > 0) {
                table_body.deleteRow(0);
            }
            response.forEach( entry => {
                add_to_table(entry, false);
            });
            sortRowsDate(document.getElementById("gpx-table"), 0, row_objects);
        });
}


function add_to_table(entry, sort) {
    let row = table_body.insertRow();
    row_objects[entry.ulid] = entry;
    let ulid = row.insertCell(0);
    ulid.innerHTML = entry.ulid;
    ulid.style.display = "none"; // used to identify row but don't display
    let time = row.insertCell(1);
    let datetime = new Date(entry.start_time);
    // time.innerHTML = datetime.toLocaleDateString();
    time.innerHTML = datetime.toLocaleDateString().replaceAll('/', '-');
    time.style.textAlign = "left";
    let type = row.insertCell(2);
    type.innerHTML = entry._type;
    type.style.textAlign = "left";
    let name = row.insertCell(3);
    name.innerHTML = entry.name;
    name.style.textAlign = "left";
    let distance = row.insertCell(4);
    distance.innerHTML = (entry.distance / 1000).toFixed(2);
    let avgVel = row.insertCell(5);
    if (entry.avg_vel === null) {
        entry.avg_vel = 0.0;
    }
    avgVel.innerHTML = entry.avg_vel.toFixed(2);
    let timeMoving = row.insertCell(6);
    // timeMoving.innerHTML = entry.time_moving;
    // timeMoving.innerHTML = new Date(1000 * entry.time_moving - 3600000).toTimeString().substring(0, 8);
    timeMoving.innerHTML = new Date(1000 * entry.time_moving - 3600000).toLocaleTimeString();
    let timeTotal = row.insertCell(7);
    timeTotal.innerHTML = new Date(1000 * entry.time_total - 3600000).toLocaleTimeString();
    let eleGain = row.insertCell(8);
    eleGain.innerHTML = Math.round(entry.ele_gain);
    let eleLoss = row.insertCell(9);
    eleLoss.innerHTML = Math.round(entry.ele_loss);
    let maxEle = row.insertCell(10);
    maxEle.innerHTML = Math.round(entry.ele_max);
    let minEle = row.insertCell(11);
    minEle.innerHTML = Math.round(entry.ele_min);
    let creator = row.insertCell(12);
    creator.innerHTML = entry.creator;

    row.addEventListener("click", (event) => {
        // prevent track from reloading if already selected. Only reapply map bounds.
        if (selected_rows.length === 1 && selected_rows.includes(entry.ulid)) {
            fitMapBounds();
            return;
        }
        if (!event.shiftKey) {
            clear_table_selection();
        }
        toggleRowSelection(entry);
        document.getSelection().removeAllRanges();
        
    });
    if (sort) {
        sortRowsDate(document.getElementById("gpx-table"), 0, row_objects);
    }
}

function clear_table_selection() {
    for (var ulid of selected_rows) {
        var rows = table_body.rows;
        for (var row of rows) {
            if (row.querySelectorAll("td")[0].innerHTML == ulid) {
                // row.classList.remove("selected-row");
                toggleRowSelection(row_objects[ulid]);
                break;
            }
        }
    }
    selected_rows = [];
}

var b; // bounding box
function toggleRowSelection(entry) {
    if (selected_rows.includes(entry.ulid)) {
        var rows = table_body.querySelectorAll("tr");
        rows.forEach(row => {
            if (row.querySelectorAll("td")[0].innerHTML == entry.ulid) {
                row.classList.remove("selected-row");
            }
        });
        removeTrack(entry.ulid);
        // remove from ulid array
        selected_rows = selected_rows.filter(function(e) { 
            return e != entry.ulid; 
        });
    } else {
        
        var rows = table_body.querySelectorAll("tr");
        rows.forEach(row => {
            if (row.querySelectorAll("td")[0].innerHTML == entry.ulid) {
                row.classList.add("selected-row");
            }
        });
        selected_rows.push(entry.ulid);

        invoke('load_track_display_data', { ulid: entry.ulid })
        .then(async (response) => {
            var geometries = response[1].geometry.geometries;
            var move = geometries[0];
            var pause = geometries[1];
            var unedPause = geometries[2];
            /*addMove(entry.ulid, move);
            addPause(entry.ulid, pause);
            addUnedPause(entry.ulid, unedPause);*/
            addTrack(entry.ulid, move, pause, unedPause);
            var nb = [[entry.x_min[0], entry.y_min[1]], [entry.x_max[0], entry.y_max[1]]]; // bounding box of new line
            if (selected_rows.length == 1) {
                b = nb;
            } else if (selected_rows.length > 1) {
                b = [[Math.min(nb[0][0], b[0][0]), Math.min(nb[0][1], b[0][1])], [Math.max(nb[1][0], b[1][0]), Math.max(nb[1][1], b[1][1])]];
            }
            currentMapBounds = b;
            fitMapBounds();
            /*map.fitBounds(b, {
                padding: { top: 25, bottom: 25, left: 25, right: 25 }
            });*/
            addPauseIcons(entry, response[0]);
        });

        invoke('load_elevation', { ulid: entry.ulid })
        .then(async (response) => {
            if (response == null) {
                // ERROR HANDLING
            } else {
                updateGraph(response[0]);
                elevationCoords[entry.ulid] = response[1];
                mapPositionIcon.getElement().style.display = "none";
                showMapPositionIcon = false;
            }
        });
        invoke('load_notes', { ulid: entry.ulid })
        .then(async (response) => {
            if (response !== null) {
                addTrackNotes(entry, response);
            }
        });

        addTrackIcons(entry);
    }
}

var currentMapBounds;
function fitMapBounds() {
    map.fitBounds(currentMapBounds, {
        padding: { top: 25, bottom: 25, left: 25, right: 25 }
    });
}

function sortRowsDate(table, columnIndex, row_objects) {
    var rows = table.querySelectorAll("tbody tr");
    var sel = "td:nth-child(" + (columnIndex + 1) + ")";
    var values = [];
    for (var i = 0; i < rows.length; i++) {
        values.push({ value: row_objects[rows[i].querySelector(sel).innerText],
                        row: rows[i] });
    }
    values.sort(comparatorDate);
    for (var i = 0; i < values.length; i++) {
        table.querySelector("tbody").appendChild(values[i].row);
    }
}

function comparatorDate(a, b) {
    let a_date = new Date(a.value.start_time);
    let b_date = new Date(b.value.start_time);
    return (a_date < b_date) - (a_date > b_date);
}

function setNoOverlay() {
    if (currentOverlayState === OverlayState.NoteEdit) {
        currentOverlayState = OverlayState.None;
        deactivateNoteEdit();
        deactivateOverlay();
        return;
    }
    if (currentOverlayState !== OverlayState.None) {
        currentOverlayState = OverlayState.None;
        deactivateRowEdit();
        deactivateNoteEdit();
        deactivateLoading();
        deactivateNoteDisplay();
        // TODO: deactivate all other possible overlay menus
        deactivateOverlay();
        deactivateAddNote();
        // activateMenuBar();
    }
}

function setEditRowOverlay() {
    if (currentOverlayState === OverlayState.None) {
        currentOverlayState = OverlayState.RowEdit;
        // deactivateMenuBar();
        activateOverlay();
        activateRowEdit();
    }
}

function setEditNoteOverlay() {
    if (currentOverlayState === OverlayState.AddNote) {
        currentOverlayState = OverlayState.NoteEdit;
        deactivateAddNote();
        activateOverlay();
        activateNoteEdit();
    }
}

function setDisplayNoteOverlay() {
    if (currentOverlayState === OverlayState.None) {
        currentOverlayState = OverlayState.DisplayNote;
    }
    activateOverlay();
    activateNoteDisplay();
}

function setLoadingInfoOverlay() {
    if (currentOverlayState === OverlayState.None) {
        currentOverlayState = OverlayState.Loading;
    }
    activateOverlay();
    activateLoading();
}

function setAddNoteOverlay() {
    if (currentOverlayState === OverlayState.None) {
        currentOverlayState = OverlayState.AddNote;
    }
    activateAddNote();
    activateOverlay();
}

function activateOverlay() {
    let overlay = document.getElementById("overlay");
    overlay.style.display = "flex";
}

function deactivateOverlay() {
    let overlay = document.getElementById("overlay");
    overlay.style.display = "none";
}

function deactivateMenuBar() {
    let bRowEdit = document.getElementById("table-button-edit");
    bRowEdit.style.opacity = "0.6";
    bRowEdit.style.pointerEvents = "none";

    let bTableOnly = document.getElementById("table-only");
    bTableOnly.style.opacity = "0.6";
    bTableOnly.style.pointerEvents = "none";

    let bTableMap = document.getElementById("table-map");
    bTableMap.style.opacity = "0.6";
    bTableMap.style.pointerEvents = "none";


    let bMapOnly = document.getElementById("map-only");
    bMapOnly.style.opacity = "0.6";
    bMapOnly.style.pointerEvents = "none";
}

function activateMenuBar() {
    let bRowEdit = document.getElementById("table-button-edit");
    bRowEdit.style.opacity = "";
    bRowEdit.style.pointerEvents = "";

    let bTableOnly = document.getElementById("table-only");
    bTableOnly.style.opacity = "";
    bTableOnly.style.pointerEvents = "";

    let bTableMap = document.getElementById("table-map");
    bTableMap.style.opacity = "";
    bTableMap.style.pointerEvents = "";

    let bMapOnly = document.getElementById("map-only");
    bMapOnly.style.opacity = "";
    bMapOnly.style.pointerEvents = "";
}

function activateRowEdit() {
    let rowEdit = document.getElementById('row-edit');
    rowEdit.style.display = "block";
}

function deactivateRowEdit() {
    let rowEdit = document.getElementById('row-edit');
    rowEdit.style.display = "";
}

function activateNoteEdit() {
    let noteEdit = document.getElementById('track-note-edit');
    noteEdit.style.display = "flex";
}

function deactivateNoteEdit() {
    let rowEdit = document.getElementById('track-note-edit');
    rowEdit.style.display = "";
}

function activateLoading() {
    let loading = document.getElementById('loading-info');
    loading.style.display = "block";
}

function deactivateLoading() {
    let loading = document.getElementById('loading-info');
    loading.style.display = "";
}

function activateAddNote() {
    let map = document.getElementById('map-box');
    map.style.zIndex = "8";
    let button = document.getElementById('add-note-button');
    button.classList.add('deactivated-button');
    let overlayBox = document.getElementById('map-overlay-box');
    overlayBox.style.display = "flex";
    addMovableMarker();
}

function deactivateAddNote() {
    let map = document.getElementById('map-box');
    map.style.zIndex = "";
    let button = document.getElementById('add-note-button');
    button.classList.remove('deactivated-button');
    let overlayBox = document.getElementById('map-overlay-box');
    overlayBox.style.display = "";
    removeMovableMarker();
}

function activateNoteDisplay() {
    let loading = document.getElementById('track-note-display');
    loading.style.display = "flex";
}

function deactivateNoteDisplay() {
    let loading = document.getElementById('track-note-display');
    loading.style.display = "";
}

function setDisplayState(state) {
    if (state !== currentDisplayState) {
        let bTableOnly = document.getElementById("table-only");
        bTableOnly.style.opacity = "";
        bTableOnly.style.pointerEvents = "";

        let bTableMap = document.getElementById("table-map");
        bTableMap.style.opacity = "";
        bTableMap.style.pointerEvents = "";

        let bMapOnly = document.getElementById("map-only");
        bMapOnly.style.opacity = "";
        bMapOnly.style.pointerEvents = "";

        if (state === DisplayState.Table) {
            bTableOnly.style.opacity = "0.6";
            bTableOnly.style.pointerEvents = "none";
            currentDisplayState = DisplayState.Table;
        } else if (state === DisplayState.Analysis) {
            bTableMap.style.opacity = "0.6";
            bTableMap.style.pointerEvents = "none";
            currentDisplayState = DisplayState.Analysis;
        } else if (state === DisplayState.Map) {
            bMapOnly.style.opacity = "0.6";
            bMapOnly.style.pointerEvents = "none";
            currentDisplayState = DisplayState.Map;
        }
    }
}

