const { invoke } = window.__TAURI__.core;

let driveSelect = null;
let analyzeBtn = null;
let loadingIndicator = null;
let directoriesChart = null;
let filesChart = null;

const colors = [
    '#FF6F61', '#6B5B93', '#88B04B', '#FF4500', '#92A8D1',
    '#955251', '#B565A7', '#009B77', '#D9BF77', '#FF4500',
    '#1E90FF', '#FFD700', '#6B5B93', '#88B04B', '#F7CAC9',
    '#92A8D1', '#955251', '#B565A7', '#009B77', '#D9BF77',
    '#808080',
];

let colorIndex = 0;
function getNextColor() {
    const color = colors[colorIndex];
    colorIndex = (colorIndex + 1) % colors.length;
    return color;
}

async function analyzeDrive(drivePath) {
    try {
        showLoadingSpinner();
        const storageData = await invoke('analyze_drive', { path: drivePath });
        console.log("Storage Data:", storageData);
        updateDisplay(storageData);
    } catch (error) {
        console.error("Failed to analyze drive:", error);
        showError(error);
    } finally {
        hideLoadingSpinner();
    }
}

async function populateDrives() {
    try {
        const drives = await invoke('get_drives');
        drives.forEach(drive => {
            const option = document.createElement('option');
            option.value = drive;
            option.textContent = drive;
            driveSelect.appendChild(option);
        });
    } catch (error) {
        console.error("Failed to fetch drives:", error);
        showError(error);
    }
}

window.addEventListener("DOMContentLoaded", async () => {
    driveSelect = document.querySelector('#drive-select');
    analyzeBtn = document.querySelector('#analyze-btn');
    loadingIndicator = document.getElementById('loading-indicator');

    drawPlaceholderForPieCharts();
    
    await populateDrives();

    analyzeBtn.addEventListener('click', () => {
        const selectedDrive = driveSelect.value;
        analyzeDrive(selectedDrive);
    });
    
});

function showLoadingSpinner() {
    loadingIndicator.style.display = 'block';
    analyzeBtn.style.display = 'none';
}

function hideLoadingSpinner() {
    loadingIndicator.style.display = 'none';
    analyzeBtn.style.display = 'block';
}

function updateDisplay(data) {
    updateStorageInfo(data);
    drawStorageOverview(data);
    drawDirectoriesChart(data);
    drawFilesChart(data);
}

function updateStorageInfo(data) {
    const storageInfo = document.querySelector('.storage-info');
    const totalGB = (data.total / 1024 / 1024 / 1024).toFixed(1);
    const usedGB = (data.used / 1024 / 1024 / 1024).toFixed(1);
    storageInfo.textContent = `${usedGB} GB of ${totalGB} GB Used`;
}

function showError(error) {
    const storageInfo = document.querySelector('.storage-info');
    storageInfo.textContent = `${error}`;
}

function drawStorageOverview(data) {
    const storageBar = document.getElementById('storageBar');
    const storageLegend = document.getElementById('storageLegend');

    // Clear previous chart data
    storageBar.innerHTML = '';
    storageLegend.innerHTML = '';

    // Filter out 'free', then sort by value in descending order
    const sortedCategories = [
        ...Object.entries(data.categories)
            .filter(([category, size]) => category !== 'free' && size > 0)
            .sort(([, sizeA], [, sizeB]) => sizeB - sizeA),
        ['free', data.categories.free] // Append 'free' at the end
    ];

    colorIndex = 0;
    sortedCategories.forEach(([category, size]) => {
        const color = getNextColor();

        // Generate the storage bar segment
        const segment = document.createElement('div');
        segment.className = 'storage-segment';
        segment.style.width = `${(size / data.total) * 100}%`;
        segment.style.backgroundColor = color;

        // Adjust the tooltip
        segment.addEventListener('mouseover', (e) => {
            tooltip.style.display = 'block';
            tooltip.textContent = `${category} ${formatBytes(size)}`;
        });
        
        segment.addEventListener('mousemove', (e) => {
            tooltip.style.left = `${e.pageX + 10}px`;
            tooltip.style.top = `${e.pageY + 10}px`;
        });
        
        segment.addEventListener('mouseout', () => {
            tooltip.style.display = 'none';
        });
        
        storageBar.appendChild(segment);

        // Generate the legend
        const legendItem = document.createElement('div');
        legendItem.className = 'legend-item';

        const colorBox = document.createElement('div');
        colorBox.className = 'legend-color';
        colorBox.style.backgroundColor = color;

        const labelText = document.createElement('span');
        labelText.innerText = `${category}`;

        legendItem.appendChild(colorBox);
        legendItem.appendChild(labelText);
        storageLegend.appendChild(legendItem);
    });
}

function drawDirectoriesChart(data) {
    if (directoriesChart) {
        directoriesChart.destroy();
    }

    colorIndex = 0;
    const ctx = document.getElementById('directories-chart').getContext('2d');
    const dirs = data.largest_directories;
    const otherDirsSize = data.used - dirs.reduce((sum, dir) => sum + dir.size, 0);
    const dirLabels = [...dirs.map(d => d.path.split('\\').pop()), 'Others'];
    const dirSizes = [...dirs.map(d => d.size), otherDirsSize];
    const dirPaths = [...dirs.map(d => d.path), '-'];

    directoriesChart = new Chart(ctx, {
        type: 'pie',
        dirPaths: dirPaths,
        data: {
            labels: dirLabels,
            datasets: [{
                data: dirSizes,
                backgroundColor: Array(dirSizes.length).fill().map(getNextColor),
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            const value = context.raw;
                            const label = context.label;
                            return `${label}: ${formatBytes(value)}`;
                        },
                        footer: function(context) {
                            const index = context[0].dataIndex;
                            const path = dirPaths[index];
                            return `${path}`;
                        },
                        // Return an empty string to hide the title
                        title: function(context) {
                            return '';
                        }
                    }
                }
            }
        }
    });
}

function drawFilesChart(data) {
    if (filesChart) {
        filesChart.destroy();
    }

    colorIndex = 0;
    const ctx = document.getElementById('files-chart').getContext('2d');
    const files = data.largest_files;
    const otherFilesSize = data.used - files.reduce((sum, file) => sum + file.size, 0);
    const fileLabels = [...files.map(f => f.name), 'Others'];
    const fileSizes = [...files.map(f => f.size), otherFilesSize];
    const filePaths = [...files.map(f => f.path), '-'];
    const fileCreationTimes = [...files.map(f => new Date(f.created.secs_since_epoch * 1000).toLocaleString()), '-'];
    const fileModificationTimes = [...files.map(f => new Date(f.modified.secs_since_epoch * 1000).toLocaleString()), '-'];

    filesChart = new Chart(ctx, {
        type: 'pie',
        filePaths: filePaths,
        fileCreationTimes: fileCreationTimes,
        fileModificationTimes: fileModificationTimes,
        data: {
            labels: fileLabels,
            datasets: [{
                data: fileSizes,
                backgroundColor: Array(fileSizes.length).fill().map(getNextColor),
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            return context.label;
                        },
                        footer: function(context) {
                            const index = context[0].dataIndex;
                            const size = context[0].raw;
                            const fullPath = filePaths[index];
                            const created = fileCreationTimes[index];
                            const modified = fileModificationTimes[index];

                            return `Size: ${formatBytes(size)}\nCreated: ${created}\nModified: ${modified}\nPath: ${fullPath}`;
                        },
                        // Return an empty string to hide the title
                        title: function(context) {
                            return '';
                        }
                    }
                }
            }
        }
    });
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function drawPlaceholderForPieCharts() {
    const plugin = {
        id: 'emptyDoughnut',
        afterDraw(chart, args, options) {
            const {datasets} = chart.data;
            const {color, width, radiusDecrease} = options;
            let hasData = false;

            for (let i = 0; i < datasets.length; i += 1) {
                const dataset = datasets[i];
                hasData |= dataset.data.length > 0;
            }

            if (!hasData) {
                const {chartArea: {left, top, right, bottom}, ctx} = chart;
                const centerX = (left + right) / 2;
                const centerY = (top + bottom) / 2;
                const r = Math.min(right - left, bottom - top) / 2;

                ctx.beginPath();
                ctx.lineWidth = width || 2;
                ctx.strokeStyle = color || 'rgba(255, 128, 0, 0.5)';
                ctx.arc(centerX, centerY, (r - radiusDecrease || 0), 0, 2 * Math.PI);
                ctx.stroke();
            }
        }
    };
    const data = {
        labels: [],
        datasets: [
          {
            label: 'Dataset 1',
            data: []
          }
        ]
    };
    filesChart = new Chart(document.getElementById('files-chart').getContext('2d'), {
        type: 'doughnut',
        data: data,
        options: {
          plugins: {
            emptyDoughnut: {
              color: 'rgba(255, 128, 0, 0.5)',
              width: 2,
              radiusDecrease: 20
            }
          }
        },
        plugins: [plugin]
    });
    directoriesChart = new Chart(document.getElementById('directories-chart').getContext('2d'), {
        type: 'doughnut',
        data: data,
        options: {
          plugins: {
            emptyDoughnut: {
              color: 'rgba(255, 128, 0, 0.5)',
              width: 2,
              radiusDecrease: 20
            }
          }
        },
        plugins: [plugin]
    });
}