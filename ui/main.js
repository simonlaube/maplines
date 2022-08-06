// access the pre-bundled global API functions
const invoke = window.__TAURI__.invoke;
const { emit, listen } = window.__TAURI__.event;


var map;
var table_body;
var row_objects;
var selected_rows;

window.onload = init;

function init() {
    reload_table();
    init_map();
    selected_rows = [];
}

listen("track_import", ev => {
    add_to_table(ev.payload, true);
});

function list_gpx_files() {
    invoke('list_gpx_files')
        .then((response) => console.log(response));
}

function reload_table() {
    // const table_body = document.getElementById("gpxTableBody");
    // const row_objects = {};
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
    let name = row.insertCell(2);
    name.innerHTML = entry.name;
    let type = row.insertCell(3);
    type.innerHTML = entry._type;
    let creator = row.insertCell(4);
    creator.innerHTML = entry.creator;

    row.addEventListener("click", (event) => {
        
        if (!event.shiftKey) {
            clear_table_selection();
        }
        toggle_row_selection(entry.ulid);
        document.getSelection().removeAllRanges();

        //invoke('load_geojson', { ulid: entry.ulid })
        invoke('calculate_pauses', { ulid: entry.ulid })
        .then(async (response) => {
            var geometries = response[1].geometry.geometries;
            var move = geometries[0];
            var pause = geometries[1];
            var uned_pause = geometries[2];
            var line = map.getSource('gps-line');
            line.setData(move);
            var up = map.getSource('uned-pause-line');
            up.setData(uned_pause);
            var p = map.getSource('pause-line');
            p.setData(pause);
            var bbox = [[entry.x_min[0], entry.y_min[1]], [entry.x_max[0], entry.y_max[1]]];
            map.fitBounds(bbox, {
                padding: { top: 25, bottom: 25, left: 25, right: 25 }
            });
            add_pause_icons(response[0]);
        });
        add_track_icons(entry);
        /*
        invoke('load_pauses', { ulid: entry.ulid })
        .then((response) => {
            add_pause_icons(response);
        })*/
        
    });
    if (sort) {
        sortRowsDate(document.getElementById("gpx-table"), 0, row_objects);
    }
}

function clear_table_selection() {
    for (var ulid of selected_rows) {
        var rows = table_body.rows;
        console.log(ulid);
        for (var row of rows) {
            console.log(row.querySelectorAll("td")[0].innerHTML);
            if (row.querySelectorAll("td")[0].innerHTML == ulid) {
                console.log("remove class");
                row.classList.remove("selected-row");
                break;
            }
        }
    }
    selected_rows = [];
}

function toggle_row_selection(ulid) {
    if (selected_rows.includes(ulid)) {
        console.log("includes");
        var rows = table_body.querySelectorAll("tr");
        rows.forEach(row => {
            if (row.querySelectorAll("td")[0].innerHTML == ulid) {
                row.classList.remove("selected-row");
            }
        });
        
        delete selected_rows.ulid;
    } else {
        var rows = table_body.querySelectorAll("tr");
        rows.forEach(row => {
            if (row.querySelectorAll("td")[0].innerHTML == ulid) {
                row.classList.add("selected-row");
            }
        });
        selected_rows.push(ulid);
        console.log(selected_rows);
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