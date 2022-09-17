const ctx = document.getElementById('elevation-chart').getContext("2d");
const labels = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];
const data = {
    labels: labels,
    datasets: [{
        label: 'Elevation',
        data: [65, 59, 80, 81, 56, 55, 40],
        fill: true,
        borderColor: 'rgb(75, 192, 192)',
        tension: 0.1
    }],
};
const myChart = new Chart(ctx, {
    type: 'line',
    data: data,
    options: {
        responsive:true,
        maintainAspectRatio: false,
    }
});
