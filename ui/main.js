// access the pre-bundled global API functions
const invoke = window.__TAURI__.invoke

function list_gpx_files() {
    invoke('list_gpx_files')
        .then((response) => console.log(response))
}

function load_line(fn) {
    invoke('load_line', { fileName: fn })
        .then((response) => {
            console.log(response);
            draw_gpx(response);
        })
}

function reload_table() {
    const table_body = document.getElementById("gpxTableBody");
    const row_objects = {};
    invoke('load_gps_summaries')
        .then((response) => {
            while(table_body.rows.length > 0) {
                table_body.deleteRow(0);
            }
            response.forEach( entry => {
                let row = table_body.insertRow();
                row_objects[entry.file_name] = entry;
                let file_name = row.insertCell(0);
                file_name.innerHTML = entry.file_name;
                file_name.style.display = "none";
                let time = row.insertCell(1);
                let datetime = new Date(entry.start_time);
                time.innerHTML = datetime.toLocaleDateString();
                let name = row.insertCell(2);
                name.innerHTML = entry.name;
                let type = row.insertCell(3);
                type.innerHTML = entry._type;
                let creator = row.insertCell(4);
                creator.innerHTML = entry.creator;

                row.addEventListener("click", () => {
                    invoke('load_geojson', { fileName: entry.file_name })
                    .then((response) => {
                        console.log(response);
                        var line = map.getSource('gps-line');
                        line.setData(response);
                        var bbox = [[entry.x_min[0], entry.y_min[1]], [entry.x_max[0], entry.y_max[1]]];
                        map.fitBounds(bbox, {
                            padding: { top: 25, bottom: 25, left: 25, right: 25 }
                        });
                    });
                });
            });
            sortRowsDate(document.getElementById("gpx-table"), 0, row_objects);
        });
}

var map;
window.onload = init_map;
function init_map() {
    map = new maplibregl.Map({
        container: 'map', // container id
        // style: 'https://demotiles.maplibre.org/style.json', // style URL
        style: 'maplibre-gl@2.1.9/style/normal.json',
        // style: 'https://api.maptiler.com/maps/hybrid/style.json?key=get_your_own_OpIi9ZULNHzrESv6T2vL',
        center: [0, 0], // starting position [lng, lat]
        zoom: 1 // starting zoom
    });
    map.addControl(new maplibregl.FullscreenControl());
    map.dragRotate.disable();
    map.touchZoomRotate.disableRotation();
    map.addControl(new maplibregl.NavigationControl());
    map.on('load', function () {
        map.addSource('gps-line', {
            'type': 'geojson',
            'data': {
                'type': 'Feature',
                'properties': {},
                'geometry': {
                    'type': 'LineString',
                    'coordinates': []
                }
            }
        });
        map.addLayer({
            'id': 'gps-line',
            'type': 'line',
            'source': 'gps-line',
            'layout': {
                'line-join': 'round',
                'line-cap': 'round'
            },
            'paint': {
                'line-color': '#9f2dcf',
                'line-width': 3
            }
        });
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
    console.log(a.start_time);
    return (a_date > b_date) - (a_date < b_date);
}