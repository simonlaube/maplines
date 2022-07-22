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
    invoke('load_gps_summaries')
        .then((response) => {
            while(table_body.rows.length > 0) {
                table_body.deleteRow(0);
            }
            response.forEach( entry => {
                let row = table_body.insertRow();
                let name = row.insertCell(0);
                name.innerHTML = entry.name;
                let type = row.insertCell(1);
                type.innerHTML = entry._type;
                let creator = row.insertCell(2);
                creator.innerHTML = entry.creator;
                let file_name = row.insertCell(3);
                file_name.innerHTML = entry.file_name;

                row.addEventListener("click", () => {
                    invoke('load_geojson', { fileName: entry.file_name })
                    .then((response) => {
                        console.log(response);
                        var line = map.getSource('gps-line');
                        line.setData(response);
                        var bbox = [[entry.x_min[0], entry.y_min[1]], [entry.x_max[0], entry.y_max[1]]];
                        map.fitBounds(bbox, {
                            padding: { top: 10, bottom: 10, left: 10, right: 10 }
                        });
                    });
                });
            });
        });
}

var map;
window.onload = init_map;
function init_map() {
    map = new maplibregl.Map({
        container: 'map', // container id
        // style: 'https://demotiles.maplibre.org/style.json', // style URL
        style: 'maplibre-gl@2.1.9/style/normal.json',
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