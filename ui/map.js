
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
    });
}

function addMove(entry, geom) {
    map.addSource(entry.ulid + ' gps-line', {
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
        'id': entry.ulid + ' gps-line',
        'type': 'line',
        'source': entry.ulid + ' gps-line',
        'layout': {
            'line-join': 'round',
            'line-cap': 'round'
        },
        'paint': {
            'line-color': '#9f2dcf',
            'line-width': 3
        }
    });
    var line = map.getSource(entry.ulid + " gps-line");
    line.setData(geom);
}

function addPause(entry, geom) {
    map.addSource(entry.ulid + ' pause-line', {
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
        'id': entry.ulid + ' pause-line',
        'type': 'line',
        'source': entry.ulid + ' pause-line',
        'layout': {
            'line-join': 'round',
            'line-cap': 'round'
        },
        'paint': {
            'line-color': '#f80',
            'line-width': 3,
            'line-dasharray': [2, 2]
        }
    });
    var line = map.getSource(entry.ulid + " pause-line");
    line.setData(geom);
}

function addUnedPause(entry, geom) {
    map.addSource(entry.ulid + ' uned-pause-line', {
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
        'id': entry.ulid + ' uned-pause-line',
        'type': 'line',
        'source': entry.ulid + ' uned-pause-line',
        'layout': {
            'line-join': 'round',
            'line-cap': 'round'
        },
        'paint': {
            'line-color': '#9f2dcf',
            'line-width': 3,
            'line-opacity': 0.2
        }
    });
    var line = map.getSource(entry.ulid + " uned-pause-line");
    line.setData(geom);
}

function removeMove(entry) {
    map.removeLayer(entry.ulid + ' gps-line');
    map.removeSource(entry.ulid + ' gps-line');
}

function removePause(entry) {
    map.removeLayer(entry.ulid + ' pause-line');
    map.removeSource(entry.ulid + ' pause-line');
}

function removePauseUned(entry) {
    map.removeLayer(entry.ulid + ' uned-pause-line');
    map.removeSource(entry.ulid + ' uned-pause-line');
}

function addTrackIcons(entry) {
    var icons = {
        'type': 'FeatureCollection',
        'features': [
            {
                'type': 'Feature',
                'properties': {
                    'message': 'Start',
                    'iconSize': [20, 20],
                    'img': "url('icons/play-button.png')"
                },
                'geometry': {
                    'type': 'Point',
                    'coordinates': [entry.start_coords[0], entry.start_coords[1]]
                }
            },
            {
                'type': 'Feature',
                'properties': {
                    'message': 'End',
                    'iconSize': [20, 20],
                    'img': "url('icons/stop-button.png')"
                },
                'geometry': {
                    'type': 'Point',
                    'coordinates': [entry.end_coords[0], entry.end_coords[1]]
                }
            },
        ]
    }
    icons.features.forEach(function (marker) {
        // create a DOM element for the marker
        var el = document.createElement('div');
        el.className = 'track-icon' + entry.ulid;
        el.classList.add('track-icon');
        el.style.backgroundImage = marker.properties.img;
        el.style.width = marker.properties.iconSize[0] + 'px';
        el.style.height = marker.properties.iconSize[1] + 'px';
            
        el.addEventListener('click', function () {
            window.alert(marker.properties.message);
        });
            
        // add marker to map
        new maplibregl.Marker(el)
        .setLngLat(marker.geometry.coordinates)
        .addTo(map);
    });
}

function addPauseIcons(entry, pauses) {
    /*document.querySelectorAll(".pause-icon").forEach(icon => {
        icon.remove();
    })*/
    pauses.forEach(p => {
        var el = document.createElement('div');
        el.className = 'track-icon' + entry.ulid;
        el.classList.add('track-icon');

        var size = 15;
        if (p.duration_sec < 600) { // Pause was shorter than 10 minutes
            size = 10;
        } else if (p.duration_sec > 3600) { // Pause was longer than 1 hour
            size = 20;
        }
        el.style.backgroundImage = "url('icons/pause-button2.png')"
        el.style.width = size + 'px';
        el.style.height = size + 'px';
            
        var date = new Date(0);
        date.setSeconds(p.duration_sec); // specify value for SECONDS here
        var timeString = date.toISOString().substr(11, 8);
        el.addEventListener('click', function () {
        window.alert(timeString);
        });
            
        // add marker to map
        new maplibregl.Marker(el)
        .setLngLat([p.coord_before[0] + (p.coord_after[0] - p.coord_before[0]) / 2, p.coord_before[1] + (p.coord_after[1] - p.coord_before[1]) / 2])
        .addTo(map);
    });
}

function removeTrackIcons(entry) {
    document.querySelectorAll(".track-icon" + entry.ulid).forEach(icon => {
        icon.remove();
    })
}