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
var myChart = new Chart(ctx, {
    type: 'scatter',
    data: data,
    options: {
        responsive:true,
        maintainAspectRatio: false,
        showLine: true
    }
});

function range(start, end) {
    return Array(end - start + 1).fill().map((_, idx) => start + idx)
}

function updateChart(data) {
    // var labels = xData /*range(0, yData.length - 1)*/;
    var data = {
        labels: labels,
        datasets: [{
            label: 'Elevation',
            data: data,
            fill: true,
            borderColor: 'rgb(75, 192, 192)',
            tension: 0.1
        }],
    };
    myChart.data = data;
    myChart.update();

    /*
    removeData(myChart);
    console.log(yData);
    addData(myChart, range(0, yData.length - 1), yData);*/
}

function addData(chart, label, data) {
    chart.data.labels.push(label);
    chart.data.datasets.forEach((dataset) => {
        dataset.data.push(data);
    });
    chart.update();
}

function removeData(chart) {
    chart.data.labels.pop();
    chart.data.datasets.forEach((dataset) => {
        dataset.data.pop();
    });
    chart.update();
}
