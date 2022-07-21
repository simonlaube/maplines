/*window.onload = init_map;
function init_map() {
    const map = new ol.Map({
        target: 'map',
        layers: [
            new ol.layer.Tile({
                source: new ol.source.OSM()
            })
        ],
        view: new ol.View({
            center: ol.proj.fromLonLat([37.41, 8.82]),
            zoom: 4
        })
    });
}

function draw_gpx(gpxObject) {
    const vector = new VectorLayer({
        source: new VectorSource(gpxObject),
        style: function (feature) {
          return style[feature.getGeometry().getType()];
        },
    });
    map.layers.push(vector);
}*/