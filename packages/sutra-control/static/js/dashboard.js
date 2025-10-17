// Sutra Control Center Dashboard - Real-time Monitoring

let ws;
let resourceChart, storageChart;
const MAX_DATA_POINTS = 20;

// Initialize charts
function initCharts() {
    // CPU & Memory Chart
    const resourceCtx = document.getElementById('resource-chart').getContext('2d');
    resourceChart = new Chart(resourceCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'CPU %',
                    data: [],
                    borderColor: '#667eea',
                    backgroundColor: 'rgba(102, 126, 234, 0.1)',
                    tension: 0.4,
                },
                {
                    label: 'Memory %',
                    data: [],
                    borderColor: '#48bb78',
                    backgroundColor: 'rgba(72, 187, 120, 0.1)',
                    tension: 0.4,
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: { color: '#f1f5f9' }
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    max: 100,
                    ticks: { color: '#cbd5e1' },
                    grid: { color: '#334155' }
                },
                x: {
                    ticks: { color: '#cbd5e1' },
                    grid: { color: '#334155' }
                }
            }
        }
    });

    // Storage Growth Chart
    const storageCtx = document.getElementById('storage-chart').getContext('2d');
    storageChart = new Chart(storageCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: 'Storage (MB)',
                data: [],
                borderColor: '#ed8936',
                backgroundColor: 'rgba(237, 137, 54, 0.1)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: { color: '#f1f5f9' }
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: { color: '#cbd5e1' },
                    grid: { color: '#334155' }
                },
                x: {
                    ticks: { color: '#cbd5e1' },
                    grid: { color: '#334155' }
                }
            }
        }
    });
}

// Connect to WebSocket
function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    
    ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
        console.log('WebSocket connected');
        updateConnectionStatus(true);
    };
    
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        updateDashboard(data);
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateConnectionStatus(false);
    };
    
    ws.onclose = () => {
        console.log('WebSocket disconnected');
        updateConnectionStatus(false);
        // Reconnect after 3 seconds
        setTimeout(connectWebSocket, 3000);
    };
}

// Update connection status indicator
function updateConnectionStatus(connected) {
    const statusDot = document.getElementById('ws-status');
    const statusText = document.getElementById('ws-text');
    
    if (connected) {
        statusDot.className = 'status-dot connected';
        statusText.textContent = 'Connected';
    } else {
        statusDot.className = 'status-dot disconnected';
        statusText.textContent = 'Disconnected';
    }
}

// Update dashboard with new data
function updateDashboard(data) {
    updateMetrics(data.metrics);
    updateComponents(data.components);
    updateCharts(data.metrics);
    updateLastUpdateTime();
}

// Update system metrics
function updateMetrics(metrics) {
    document.getElementById('storage-size').textContent = `${metrics.storage_size_mb} MB`;
    document.getElementById('total-concepts').textContent = metrics.total_concepts.toLocaleString();
    document.getElementById('total-associations').textContent = metrics.total_associations.toLocaleString();
    document.getElementById('cpu-usage').textContent = `${metrics.cpu_percent}%`;
    document.getElementById('memory-usage').textContent = `${metrics.memory_percent}%`;
}

// Update component cards
function updateComponents(components) {
    const grid = document.getElementById('components-grid');
    grid.innerHTML = '';
    
    Object.entries(components).forEach(([key, component]) => {
        const card = createComponentCard(key, component);
        grid.appendChild(card);
    });
}

// Create component card element
function createComponentCard(key, component) {
    const card = document.createElement('div');
    card.className = 'component-card';
    
    const uptime = component.uptime ? formatUptime(component.uptime) : 'N/A';
    const cpu = component.cpu_percent !== null ? `${component.cpu_percent.toFixed(1)}%` : 'N/A';
    const memory = component.memory_mb !== null ? `${component.memory_mb.toFixed(1)} MB` : 'N/A';
    
    card.innerHTML = `
        <div class="component-header">
            <div class="component-name">${component.name}</div>
            <span class="component-badge ${component.state}">${component.state}</span>
        </div>
        <div class="component-stats">
            <div class="stat-item">
                <div class="stat-label">Uptime</div>
                <div class="stat-value">${uptime}</div>
            </div>
            <div class="stat-item">
                <div class="stat-label">CPU</div>
                <div class="stat-value">${cpu}</div>
            </div>
            <div class="stat-item">
                <div class="stat-label">Memory</div>
                <div class="stat-value">${memory}</div>
            </div>
            <div class="stat-item">
                <div class="stat-label">PID</div>
                <div class="stat-value">${component.pid || 'N/A'}</div>
            </div>
        </div>
        ${component.error ? `<div class="error-message" style="color: var(--error); font-size: 0.85rem; margin-bottom: 1rem;">${component.error}</div>` : ''}
        <div class="component-actions">
            <button class="btn btn-start" onclick="controlComponent('${key}', 'start')" ${component.state === 'running' ? 'disabled' : ''}>
                Start
            </button>
            <button class="btn btn-stop" onclick="controlComponent('${key}', 'stop')" ${component.state === 'stopped' ? 'disabled' : ''}>
                Stop
            </button>
            <button class="btn btn-restart" onclick="controlComponent('${key}', 'restart')" ${component.state === 'stopped' ? 'disabled' : ''}>
                Restart
            </button>
        </div>
    `;
    
    return card;
}

// Control component (start/stop/restart)
async function controlComponent(component, action) {
    try {
        const response = await fetch(`/api/components/${component}/${action}`, {
            method: 'POST'
        });
        const result = await response.json();
        
        if (!result.success) {
            alert(`Failed to ${action} ${component}: ${result.error || 'Unknown error'}`);
        }
    } catch (error) {
        console.error(`Error ${action}ing ${component}:`, error);
        alert(`Error ${action}ing ${component}`);
    }
}

// Update charts with new data
function updateCharts(metrics) {
    const timestamp = new Date(metrics.timestamp).toLocaleTimeString();
    
    // Update resource chart
    if (resourceChart.data.labels.length >= MAX_DATA_POINTS) {
        resourceChart.data.labels.shift();
        resourceChart.data.datasets[0].data.shift();
        resourceChart.data.datasets[1].data.shift();
    }
    
    resourceChart.data.labels.push(timestamp);
    resourceChart.data.datasets[0].data.push(metrics.cpu_percent);
    resourceChart.data.datasets[1].data.push(metrics.memory_percent);
    resourceChart.update('none'); // No animation for smoother updates
    
    // Update storage chart
    if (storageChart.data.labels.length >= MAX_DATA_POINTS) {
        storageChart.data.labels.shift();
        storageChart.data.datasets[0].data.shift();
    }
    
    storageChart.data.labels.push(timestamp);
    storageChart.data.datasets[0].data.push(metrics.storage_size_mb);
    storageChart.update('none');
}

// Format uptime in human-readable format
function formatUptime(seconds) {
    if (seconds < 60) {
        return `${Math.floor(seconds)}s`;
    } else if (seconds < 3600) {
        return `${Math.floor(seconds / 60)}m`;
    } else if (seconds < 86400) {
        const hours = Math.floor(seconds / 3600);
        const mins = Math.floor((seconds % 3600) / 60);
        return `${hours}h ${mins}m`;
    } else {
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        return `${days}d ${hours}h`;
    }
}

// Update last update timestamp
function updateLastUpdateTime() {
    const now = new Date().toLocaleTimeString();
    document.getElementById('last-update').textContent = now;
}

// Initialize dashboard
document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing Sutra Control Center Dashboard...');
    initCharts();
    connectWebSocket();
});
