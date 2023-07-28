var trackNotes = [];
function initMap() {
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
}

var curr_move_line = [];
var curr_uned_pause_line = [];
var curr_pause_line = [];
var movable_marker_exists = false;

function addTrack(ulid, move, pause, uned_pause) {
    addMove(ulid, move);
    addPause(ulid, pause);
    addUnedPause(ulid, uned_pause);
    curr_move_line.push([ulid, move]);
    curr_pause_line.push([ulid, pause]);
    curr_uned_pause_line.push([ulid, uned_pause]);
}

function addMove(ulid, geom) {
    map.addSource(ulid + ' gps-line', {
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
        'id': ulid + ' gps-line',
        'type': 'line',
        'source': ulid + ' gps-line',
        'layout': {
            'line-join': 'round',
            'line-cap': 'round'
        },
        'paint': {
            'line-color': '#9f2dcf',
            'line-width': 3
        }
    });
    var line = map.getSource(ulid + " gps-line");
    line.setData(geom);
}

function addPause(ulid, geom) {
    map.addSource(ulid + ' pause-line', {
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
        'id': ulid + ' pause-line',
        'type': 'line',
        'source': ulid + ' pause-line',
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
    var line = map.getSource(ulid + " pause-line");
    line.setData(geom);
}

function addUnedPause(ulid, geom) {
    map.addSource(ulid + ' uned-pause-line', {
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
        'id': ulid + ' uned-pause-line',
        'type': 'line',
        'source': ulid + ' uned-pause-line',
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
    var line = map.getSource(ulid + " uned-pause-line");
    line.setData(geom);
}

function removeMove(ulid) {
    map.removeLayer(ulid + ' gps-line');
    map.removeSource(ulid + ' gps-line');
}

function removePause(ulid) {
    map.removeLayer(ulid + ' pause-line');
    map.removeSource(ulid + ' pause-line');
}

function removePauseUned(ulid) {
    map.removeLayer(ulid + ' uned-pause-line');
    map.removeSource(ulid + ' uned-pause-line');
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
        el.style.backgroundImage = "url('icons/pause-button2.png')";
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

function addTrackNotes(entry, notes) {
    notes.forEach(n => {
        var el = document.createElement('div');
        el.className = 'track-note' + entry.ulid;
        el.classList.add('track-note');

        var size = 15;
        if (n.icon === "Picture") {
            el.style.backgroundImage = "url('icons/picture.png')";
        } else if (n.icon === "Text") {
            el.style.backgroundImage = "url('icons/text.png')";
        } else {
            el.style.backgroundImage = "url('icons/info.png')";
        }
        el.style.width = size + 'px';
        el.style.height = size + 'px';
        el.addEventListener('click', function () {
            // TODO: open note display
            displayNote(n);
        });
            
        // add marker to map
        new maplibregl.Marker(el)
        .setLngLat(n.coords)
        .addTo(map);
    });
}

function removeTrackIcons(ulid) {
    document.querySelectorAll(".track-icon" + ulid).forEach(icon => {
        icon.remove();
    })
}

function removeTrack(ulid) {
    removeMove(ulid);
    removePause(ulid);
    removePauseUned(ulid);
    removeTrackIcons(ulid);
    // remove ulid and geometry from cache
    for (i = 0; i < curr_move_line.length; i++) {
        if (curr_move_line[i][0] === ulid) {
            curr_move_line.pop(i);
            curr_pause_line.pop(i);
            curr_uned_pause_line.pop(i);
        }
    }
}

function toggleMapStyle() {
    for (u of selected_rows) {
        removeMove(u);
        removePause(u);
        removePauseUned(u);
    }

    // Setting the style of the map removes all layers (including user created ones).
    // Therefore user layers have to be added again after style change.
    // The timeout quick-fixes a problem where the layers are added again before
    // the style change removes them (asynchronosity).
    if (document.getElementById('satellite').innerHTML === 'satellite') {
        map.setStyle('https://api.maptiler.com/maps/hybrid/style.json?key=get_your_own_OpIi9ZULNHzrESv6T2vL');
        document.getElementById('satellite').innerHTML = 'vector';
    } else {
        map.setStyle('maplibre-gl@2.1.9/style/normal.json');
        document.getElementById('satellite').innerHTML = 'satellite';
    }

    setTimeout(function() {
        for ([u, geom] of curr_move_line) {
            addMove(u, geom);
        }
        for ([u, geom] of curr_pause_line) {
            addPause(u, geom);
        }
        for ([u, geom] of curr_uned_pause_line) {
            addUnedPause(u, geom);
        }
        if (movable_marker_exists === true) {
            map.addSource('point', {
                'type': 'geojson',
                'data': movableMarker
            });
                 
            map.addLayer({
                'id': 'point',
                'type': 'circle',
                'source': 'point',
                'paint': {
                'circle-radius': 10,
                'circle-color': '#3887be'
                }
            });
        }
    }, 150);
}

async function switchBaseMap(map, styleID) {
    const response = await fetch(styleID);
    const responseJson = await response.json();
    const newStyle = responseJson;
  
    const currentStyle = map.getStyle();
    // ensure any sources from the current style are copied across to the new style
    newStyle.sources = Object.assign({},
        currentStyle.sources,
        newStyle.sources
    );
  
    // find the index of where to insert our layers to retain in the new style
    let labelIndex = newStyle.layers.findIndex((el) => {
        return false;
        return el.id == 'waterway-label';
    });
  
    // default to on top
    if (labelIndex === -1) {
        labelIndex = newStyle.layers.length;
    }  
    const appLayers = currentStyle.layers.filter((el) => {
      // app layers are the layers to retain, and these are any layers which have a different source set
        return (
            el.source &&
            el.source != 'mapbox://mapbox.satellite' &&
            el.source != 'mapbox' &&
            el.source != 'composite'
        );
    });
    newStyle.layers = [
        ...newStyle.layers.slice(0, labelIndex),
        ...appLayers,
        ...newStyle.layers.slice(labelIndex, -1),
    ];
    map.setStyle(newStyle);
}

var showMapPositionIcon = true;
var moveMapPositionIcon = false;

function initMapPositionIcon() {
    var el = document.createElement('div');
    el.className = 'map-position-icon';

    var size = 15;
    el.style.backgroundImage = "url('icons/map-position-icon.png')";
    el.style.width = size + 'px';
    el.style.height = size + 'px';
    el.style.display = "none";

    mapPositionIcon = new maplibregl.Marker(el)
        .setLngLat([0, 0])
        .addTo(map);
    
    document.getElementById('elevation-graph').onclick = function(e) {
        if (moveMapPositionIcon) {
            moveMapPositionIcon = false;
        } else {
            moveMapPositionIcon = true;
        }
    }

    document.getElementById('elevation-graph').onmouseenter = function(e) {
        mapPositionIcon.getElement().style.display = "";
        showMapPositionIcon = true;
        moveMapPositionIcon = true;
    }
    document.getElementById('elevation-graph').onmousemove = function(e) {
        let pos = elevationGraph.getSelection();
        if (showMapPositionIcon && moveMapPositionIcon && pos !== -1 && selected_rows.length > 0) {
            let long = elevationCoords[selected_rows[0]][pos][0];
            let lat = elevationCoords[selected_rows[0]][pos][1];
            updateMapPositionIcon(long, lat);
        }
    }
}

function updateMapPositionIcon(long, lat) {
    // mapPositionIcon.getElement().style.display = "block";
    mapPositionIcon.setLngLat(new maplibregl.LngLat(long, lat));
}



// --------------------- Movable Marker ---------------------

var canvas;
var movableMarker;

function onMove(e) {
    var coords = e.lngLat;
    
    // Set a UI indicator for dragging.
    canvas.style.cursor = 'grabbing';
    
    // Update the Point feature in `geojson` coordinates
    // and call setData to the source layer `point` on it.
    movableMarker.features[0].geometry.coordinates = [coords.lng, coords.lat];
    map.getSource('point').setData(movableMarker);
}

function onUp(e) {
    canvas.style.cursor = '';

    
    // Unbind mouse/touch events
    map.off('mousemove', onMove);
    map.off('touchmove', onMove);
}

function addMovableMarker() {

    canvas = map.getCanvasContainer();
    movableMarker = {
        'type': 'FeatureCollection',
        'features': [
            {
                'type': 'Feature',
                'geometry': {
                    'type': 'Point',
                    'coordinates': [map.getCenter().lng, map.getCenter().lat]
                }
            }
        ]
    };
    map.addSource('point', {
        'type': 'geojson',
        'data': movableMarker
    });
         
    map.addLayer({
        'id': 'point',
        'type': 'circle',
        'source': 'point',
        'paint': {
        'circle-radius': 10,
        'circle-color': '#3887be'
        }
    });
         
    // When the cursor enters a feature in the point layer, prepare for dragging.
    map.on('mouseenter', 'point', function () {
        map.setPaintProperty('point', 'circle-color', '#3bb2d0');
        canvas.style.cursor = 'move';
    });
        
    map.on('mouseleave', 'point', function () {
        map.setPaintProperty('point', 'circle-color', '#3887be');
        canvas.style.cursor = '';
    });
        
    map.on('mousedown', 'point', function (e) {
        // Prevent the default map drag behavior.
        e.preventDefault();
        
        canvas.style.cursor = 'grab';
        
        map.on('mousemove', onMove);
        map.once('mouseup', onUp);
    });
        
    map.on('touchstart', 'point', function (e) {
        if (e.points.length !== 1) return;
        
        // Prevent the default map drag behavior.
        e.preventDefault();
        
        map.on('touchmove', onMove);
        map.once('touchend', onUp);
    });
    movable_marker_exists = true;
}

function removeMovableMarker() {
    map.off('mousemove', onMove);
    map.off('touchmove', onMove);
    map.removeLayer('point');
    map.removeSource('point');
    movable_marker_exists = false;
}