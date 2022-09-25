// access the pre-bundled global API functions
const invoke = window.__TAURI__.invoke;
const { emit, listen } = window.__TAURI__.event;

var map;
var table_body;
var row_objects;
var selected_rows;

const DisplayState = {
    Table: 'Table',
    Analysis: 'Analysis',
    Map: 'Map',
}
var currentDisplayState = DisplayState.Analysis;

const OverlayState = {
    None: 'None',
    RowEdit: 'RowEdit',
    Settings: 'Settings',
};
var currentOverlayState = OverlayState.None;

window.onload = init;

function init() {
    initContentResize();
    reload_table();
    init_map();
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
    console.log("drag start");
    document.getElementById('content-wrapper').style.cursor = "ns-resize";
    document.getElementById('content-wrapper').style.pointerEvents = "none";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "ns-resize";
    window.addEventListener('mouseup', onHDragEnd);
    window.addEventListener('selectstart', disableSelect);
    window.addEventListener('mousemove', moveHSeparator);
    
}

function onHDragEnd() {
    console.log("drag end");
    window.removeEventListener('mouseup', onHDragEnd);
    window.removeEventListener('selectstart', disableSelect);
    window.removeEventListener('mousemove', moveHSeparator);
    document.getElementById('content-wrapper').style.cursor = "";
    document.getElementById('content-wrapper').style.pointerEvents = "";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "";
    map.resize();
}

function moveHSeparator(e) {
    let percentTop = e.clientY / e.view.innerHeight * 100;
    let content = document.getElementById("content-wrapper");
    content.style.gridAutoRows = percentTop + "% 0.15rem auto";
    map.resize();
}

function startVDrag(event) {
    console.log("drag start");
    document.getElementById('content-wrapper').style.cursor = "ew-resize";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "ew-resize";
    window.addEventListener('mouseup', onVDragEnd);
    window.addEventListener('selectstart', disableSelect);
    window.addEventListener('mousemove', moveVSeparator);
    
}

function onVDragEnd() {
    console.log("drag end");
    window.removeEventListener('mouseup', onVDragEnd);
    window.removeEventListener('selectstart', disableSelect);
    window.removeEventListener('mousemove', moveVSeparator);
    document.getElementById('content-wrapper').style.cursor = "";
    document.getElementsByClassName('maplibregl-canvas')[0].style.cursor = "";
    map.resize();
}

function moveVSeparator(e) {
    let percentLeft = e.clientX / e.view.innerWidth * 100;
    console.log(e.clientX);
    let content = document.getElementById("content-wrapper");
    content.style.gridAutoColumns = percentLeft + "% 0.15rem auto";
    map.resize();
}

function setCursorStyle() {
    document.getElementById("content-wrapper").style.cursor = "wait";
}

function list_gpx_files() {
    invoke('list_gpx_files')
        .then((response) => console.log(response));
}

function reload_table() {
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
    time.innerHTML = datetime.toLocaleDateString();
    let type = row.insertCell(2);
    type.innerHTML = entry._type;
    let name = row.insertCell(3);
    name.innerHTML = entry.name;
    let distance = row.insertCell(4);
    distance.innerHTML = (entry.distance / 1000).toFixed(2);
    let creator = row.insertCell(5);
    creator.innerHTML = entry.creator;

    row.addEventListener("click", (event) => {
        
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
            console.log(row.querySelectorAll("td")[0].innerHTML);
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
        removeMove(entry);
        removePause(entry);
        removePauseUned(entry);
        removeTrackIcons(entry);
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
            addMove(entry, move);
            addPause(entry, pause);
            addUnedPause(entry, unedPause);
            var nb = [[entry.x_min[0], entry.y_min[1]], [entry.x_max[0], entry.y_max[1]]]; // bounding box of new line
            if (selected_rows.length == 1) {
                b = nb;
            } else if (selected_rows.length > 1) {
                b = [[Math.min(nb[0][0], b[0][0]), Math.min(nb[0][1], b[0][1])], [Math.max(nb[1][0], b[1][0]), Math.max(nb[1][1], b[1][1])]];
            }
            map.fitBounds(b, {
                padding: { top: 25, bottom: 25, left: 25, right: 25 }
            });
            addPauseIcons(entry, response[0]);
        });

        invoke('load_elevation', { ulid: entry.ulid })
        .then(async (response) => {
            if (response == null) {
                // ERROR HANDLING
            } else {
                console.log(response);
                updateChart(response[0]);
                addData(response[1]);
            }
        });

        addTrackIcons(entry);
    }
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
    if (currentOverlayState !== OverlayState.None) {
        currentOverlayState = OverlayState.None;
        deactivateRowEdit();
        // TODO: deactivate all other possible overlay menus
        deactivateOverlay();
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

