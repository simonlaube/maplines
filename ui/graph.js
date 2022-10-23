var elevationGraph = new Dygraph(
    document.getElementById('elevation-graph'),
    "X,Y\n" +
    "0, 0\n",
    {
        showRangeSelector : false,
        panEdgeFraction : 0.05,
        gridLineColor : "#ccc",
        colors : [ "#9f2dcf" ],
        strokeWidth : 1.0,
        fillGraph : true,
    }
);
elevationGraph.resize();

// elevationGraph.on('mouseover', console.log("over graph"));
/*
elevationGraph.on('mousemove', function() {
    let long = elevationCoords[selected_rows[0]][1];
    let lat = elevationCoords[selected_rows[0]][0];
    showMapPositionIcon(long, lat);
    console.log(lat + " : " + long);
});*/
/*
const ctx = document.getElementById('elevation-chart').getContext("2d");
var labels = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];
var data = {
    labels: labels,
    datasets: [{
        label: 'Elevation',
        data: [65, 59, 80, 81, 56, 55, 40],
        fill: true,
        borderColor: 'rgb(75, 192, 192)',
        tension: 0.1
    }],
};
var graphChart = new Chart(ctx, {
    type: 'scatter',
    data: data,
    options: {
        responsive:true,
        maintainAspectRatio: false,
        showLine: true,
        plugins: {
            zoom: {
                zoom: {
                    wheel: {
                        enabled: true,
                        modifierKey: 'shift',
                    },
                    pinch: {
                        enabled: true,
                    },
                    mode: 'x',
                },
                pan: {
                    enabled: true,
                    mode: 'xy',
                }
            }
        },
        elements: {
            point: {
                borderWidth: 0,
                radius: 0,
                backgroundColor: 'rgba(0,0,0,0)'
            }
        }
    }
});
*/
function range(start, end) {
    return Array(end - start + 1).fill().map((_, idx) => start + idx)
}

function updateGraph(data) {
    // var labels = xData /*range(0, yData.length - 1)*/;
    /*var data = {
        labels: labels,
        datasets: [{
            label: 'Elevation',
            data: data,
            fill: true,
            borderColor: 'rgb(75, 192, 192)',
            tension: 0.1
        }],
    };*/
    // graphChart.data = data;
    // graphChart.update();
    elevationGraph.updateOptions({
        file: data,
    });
    elevationGraph.resize();

    /*
    removeData(myChart);
    console.log(yData);
    addData(myChart, range(0, yData.length - 1), yData);*/
}

function addData(data) {
    // graphChart.data.labels.push(label);
    graphChart.data.datasets.push({
        label: 'Smoothed',
        data: data,
        fill: true,
        borderColor: '#9f2dcf',
        tension: 0.1
    })
    /*
    chart.data.datasets.forEach((dataset) => {
        dataset.data.push(data);
    });*/
    graphChart.update();
}

function removeData(chart) {
    chart.data.labels.pop();
    chart.data.datasets.forEach((dataset) => {
        dataset.data.pop();
    });
    chart.update();
}
