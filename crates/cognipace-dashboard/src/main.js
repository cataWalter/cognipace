import { invoke } from '@tauri-apps/api/tauri';

// State
let cognitiveState = null;
let config = null;
let stats = null;

// DOM Elements
const app = document.getElementById('app');

// State emoji mapping
const stateEmojis = {
    deep_focus: '🟢',
    flow_building: '🟡',
    fragmented: '🟠',
    fatigued: '🔴',
    overloaded: '⚫',
    recovering: '🔵',
};

// State color mapping
const stateColors = {
    deep_focus: '#4caf50',
    flow_building: '#ffeb3b',
    fragmented: '#ff9800',
    fatigued: '#f44336',
    overloaded: '#9e9e9e',
    recovering: '#2196f3',
};

// Format state name
function formatStateName(name) {
    return name.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

// Get gauge color based on load
function getGaugeColor(load) {
    if (load < 0.3) return '#4caf50';
    if (load < 0.5) return '#ffeb3b';
    if (load < 0.7) return '#ff9800';
    if (load < 0.9) return '#f44336';
    return '#9e9e9e';
}

// Render the dashboard
function render() {
    if (!cognitiveState || cognitiveState.status === 'waiting') {
        app.innerHTML = `
            <div class="dashboard">
                <div class="header">
                    <div class="logo">
                        <span>🧠</span>
                        <h1>CogniPace Dashboard</h1>
                    </div>
                </div>
                <div class="waiting">
                    <div class="spinner"></div>
                    <p>Waiting for sensor data...</p>
                    <p style="font-size: 0.8rem; margin-top: 0.5rem;">Ensure sensors are enabled and permissions granted</p>
                </div>
            </div>
        `;
        return;
    }

    const state = cognitiveState;
    const stateName = state.attention_state;
    const emoji = stateEmojis[stateName] || '⚪';
    const color = stateColors[stateName] || '#a0a0a0';
    const loadPercent = Math.round(state.load_score * 100);
    const gaugeColor = getGaugeColor(state.load_score);

    app.innerHTML = `
        <div class="dashboard">
            <div class="header">
                <div class="logo">
                    <span>🧠</span>
                    <h1>CogniPace Dashboard</h1>
                </div>
                <div class="actions">
                    <button class="btn btn-secondary" onclick="refreshData()">Refresh</button>
                    <button class="btn btn-secondary" onclick="purgeData()">Purge Data</button>
                </div>
            </div>

            <div class="grid">
                <!-- Current State Card -->
                <div class="card">
                    <h2>Current State</h2>
                    <div class="state-display">
                        <div class="state-emoji">${emoji}</div>
                        <div class="state-name" style="color: ${color}">${formatStateName(stateName)}</div>
                        <div class="state-description">${state.attention_state_description || 'Analyzing...'}</div>
                    </div>
                </div>

                <!-- Cognitive Load Gauge -->
                <div class="card">
                    <h2>Cognitive Load</h2>
                    <div class="gauge-container">
                        <div class="gauge" style="--gauge-color: ${gaugeColor}; --gauge-percent: ${loadPercent}%">
                            <div class="gauge-value">${loadPercent}%</div>
                        </div>
                    </div>
                    <div class="recommendation">
                        <p><strong>Load Breakdown:</strong></p>
                        <p>Intrinsic: ${Math.round(state.intrinsic_load * 100)}% | Extraneous: ${Math.round(state.extraneous_load * 100)}% | Physiological: ${Math.round(state.physiological_load * 100)}%</p>
                    </div>
                </div>

                <!-- Recommendation Card -->
                <div class="card">
                    <h2>Recommendation</h2>
                    <div class="recommendation">
                        <p>${getRecommendation(stateName)}</p>
                    </div>
                    ${state.forecast ? `
                        <div style="margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--bg-secondary);">
                            <p style="font-size: 0.85rem; color: var(--text-secondary);">
                                <strong>Forecast:</strong> ${Math.round(state.forecast.predicted_load * 100)}% load predicted in ${Math.round(state.forecast.horizon_seconds / 60)}min
                                (${Math.round(state.forecast.overload_probability * 100)}% overload probability)
                            </p>
                        </div>
                    ` : ''}
                </div>

                <!-- Stats Card -->
                <div class="card">
                    <h2>Statistics</h2>
                    <div class="stats-grid">
                        <div class="stat-item">
                            <div class="stat-value">${stats?.feature_count || 0}</div>
                            <div class="stat-label">Data Points</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value">${stats?.state_count || 0}</div>
                            <div class="stat-label">States Recorded</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value">${stats?.adaptation_count || 0}</div>
                            <div class="stat-label">Adaptations</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value">${stats?.retention_days || 7}</div>
                            <div class="stat-label">Retention (days)</div>
                        </div>
                    </div>
                </div>

                <!-- Sensors Card -->
                <div class="card">
                    <h2>Sensors</h2>
                    <div class="sensors-list">
                        <div class="sensor-item">
                            <span>👁️ Eye Tracking</span>
                            <div class="sensor-status">
                                <div class="status-dot ${config?.sensors?.eye_tracking?.enabled ? 'active' : 'inactive'}"></div>
                                <span>${config?.sensors?.eye_tracking?.enabled ? 'Enabled' : 'Disabled'}</span>
                            </div>
                        </div>
                        <div class="sensor-item">
                            <span>⌨️ Keystroke Dynamics</span>
                            <div class="sensor-status">
                                <div class="status-dot ${config?.sensors?.keystroke?.enabled ? 'active' : 'inactive'}"></div>
                                <span>${config?.sensors?.keystroke?.enabled ? 'Enabled' : 'Disabled'}</span>
                            </div>
                        </div>
                        <div class="sensor-item">
                            <span>📱 App Usage</span>
                            <div class="sensor-status">
                                <div class="status-dot ${config?.sensors?.app_usage?.enabled ? 'active' : 'inactive'}"></div>
                                <span>${config?.sensors?.app_usage?.enabled ? 'Enabled' : 'Disabled'}</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Adaptations Card -->
                <div class="card">
                    <h2>Adaptations</h2>
                    <div class="sensors-list">
                        <div class="sensor-item">
                            <span>🔔 Notifications</span>
                            <div class="sensor-status">
                                <div class="status-dot ${config?.adaptations?.notifications?.enabled ? 'active' : 'inactive'}"></div>
                                <span>${config?.adaptations?.notifications?.mode || 'adaptive'}</span>
                            </div>
                        </div>
                        <div class="sensor-item">
                            <span>🎨 UI Density</span>
                            <div class="sensor-status">
                                <div class="status-dot ${config?.adaptations?.ui_density?.enabled ? 'active' : 'inactive'}"></div>
                                <span>${config?.adaptations?.ui_density?.enabled ? 'Auto' : 'Manual'}</span>
                            </div>
                        </div>
                        <div class="sensor-item">
                            <span>☕ Break Scheduler</span>
                            <div class="sensor-status">
                                <div class="status-dot ${config?.adaptations?.breaks?.enabled ? 'active' : 'inactive'}"></div>
                                <span>Every ${config?.adaptations?.breaks?.interval_minutes || 90}min</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function getRecommendation(stateName) {
    const recommendations = {
        deep_focus: "You're in deep focus! Continue your work. All non-critical notifications are being queued.",
        flow_building: "Flow state is building. Maintaining optimal conditions for concentration.",
        fragmented: "Your attention appears fragmented. Consider taking a short break or batching notifications.",
        fatigued: "Fatigue detected. A break is strongly recommended to restore cognitive capacity.",
        overloaded: "Cognitive overload detected! Emergency simplification activated. Please take a break immediately.",
        recovering: "Recovery in progress. Gradually reintroducing normal operations.",
    };
    return recommendations[stateName] || "Monitoring your cognitive state...";
}

// Refresh data from backend
async function refreshData() {
    try {
        cognitiveState = await invoke('get_cognitive_state');
        stats = await invoke('get_stats');
        config = await invoke('get_config');
        render();
    } catch (e) {
        console.error('Failed to refresh data:', e);
    }
}

// Purge all data
async function purgeData() {
    if (confirm('Are you sure you want to purge all data? This cannot be undone.')) {
        try {
            await invoke('purge_data');
            stats = await invoke('get_stats');
            render();
        } catch (e) {
            console.error('Failed to purge data:', e);
        }
    }
}

// Make functions available globally
window.refreshData = refreshData;
window.purgeData = purgeData;

// Initial load and polling
refreshData();
setInterval(refreshData, 5000); // Refresh every 5 seconds
