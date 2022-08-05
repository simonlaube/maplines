
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

function add_track_icons(entry) {
    document.querySelectorAll(".track-icon").forEach(icon => {
        icon.remove();
    })
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
        el.className = 'track-icon';
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

function add_pause_icons(pauses) {
    document.querySelectorAll(".pause-icon").forEach(icon => {
        icon.remove();
    })
    pauses.forEach(p => {
        var el = document.createElement('div');
        el.className = 'track-icon';

        var size = 15;
        if (p.duration_sec < 600) { // Pause was shorter than 10 minutes
            size = 10;
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
        .setLngLat([p.point_before[0], p.point_before[1]])
        .addTo(map);
    });
}