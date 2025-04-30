// gauges.js - Render circular gauges using Chart.js
import Chart from 'https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js';

export function createGauge(canvasId, value, label, color) {
    const ctx = document.getElementById(canvasId).getContext('2d');
    if (!ctx) return;
    return new Chart(ctx, {
        type: 'doughnut',
        data: {
            datasets: [{
                data: [value, 100 - value],
                backgroundColor: [color, '#23272f'],
                borderWidth: 0,
                cutout: '72%',
            }],
            labels: [label, '']
        },
        options: {
            plugins: {
                legend: { display: false },
                tooltip: { enabled: false },
                title: { display: false },
            },
            rotation: -90,
            circumference: 180,
            responsive: false,
        }
    });
}

window.createGauge = createGauge;
