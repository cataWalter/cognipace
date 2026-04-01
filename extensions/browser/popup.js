// CogniPace Browser Extension - Popup Script

const STATE_EMOJIS = {
  deep_focus: '🟢',
  flow_building: '🟡',
  fragmented: '🟠',
  fatigued: '🔴',
  overloaded: '⚫',
  recovering: '🔵',
};

const STATE_COLORS = {
  deep_focus: '#4caf50',
  flow_building: '#ffeb3b',
  fragmented: '#ff9800',
  fatigued: '#f44336',
  overloaded: '#9e9e9e',
  recovering: '#2196f3',
};

const STATE_DESCRIPTIONS = {
  deep_focus: 'Deep focus detected. All systems optimized for concentration.',
  flow_building: 'Flow state building. Gradually reducing interruptions.',
  fragmented: 'Attention fragmented. Consider taking a break.',
  fatigued: 'Fatigue detected. Strongly recommend a break.',
  overloaded: 'Cognitive overload! Emergency simplification activated.',
  recovering: 'Recovery in progress. Gradually re-introducing stimuli.',
};

// DOM Elements
const stateEmoji = document.getElementById('stateEmoji');
const stateName = document.getElementById('stateName');
const stateDesc = document.getElementById('stateDesc');
const loadFill = document.getElementById('loadFill');
const loadText = document.getElementById('loadText');
const status = document.getElementById('status');
const summarizeBtn = document.getElementById('summarizeBtn');
const reduceUiBtn = document.getElementById('reduceUiBtn');
const openDashboard = document.getElementById('openDashboard');

// Get current state from background
async function getState() {
  return new Promise((resolve) => {
    chrome.runtime.sendMessage({ type: 'GET_STATE' }, resolve);
  });
}

// Update UI with current state
function updateUI(state) {
  if (!state || !state.attention_state) {
    stateEmoji.textContent = '⚪';
    stateName.textContent = 'Waiting...';
    stateDesc.textContent = 'Connect to CogniPace to see your cognitive state';
    loadFill.style.width = '0%';
    loadFill.style.background = '#a0a0a0';
    loadText.textContent = '0%';
    status.textContent = 'Status: Not connected';
    return;
  }
  
  const emoji = STATE_EMOJIS[state.attention_state] || '⚪';
  const color = STATE_COLORS[state.attention_state] || '#a0a0a0';
  const desc = STATE_DESCRIPTIONS[state.attention_state] || '';
  const loadPercent = Math.round(state.load_score * 100);
  
  stateEmoji.textContent = emoji;
  stateName.textContent = formatStateName(state.attention_state);
  stateName.style.color = color;
  stateDesc.textContent = desc;
  
  loadFill.style.width = `${loadPercent}%`;
  loadFill.style.background = color;
  loadText.textContent = `${loadPercent}%`;
  
  status.textContent = `Status: Connected | Load: ${loadPercent}%`;
}

function formatStateName(name) {
  return name.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

// Event Listeners
summarizeBtn.addEventListener('click', async () => {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  chrome.tabs.sendMessage(tab.id, { type: 'SUMMARIZE' });
  window.close();
});

reduceUiBtn.addEventListener('click', async () => {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  chrome.tabs.sendMessage(tab.id, { type: 'REDUCE_UI' });
  window.close();
});

openDashboard.addEventListener('click', () => {
  chrome.tabs.create({ url: 'http://localhost:1420' });
  window.close();
});

// Initialize
chrome.runtime.onMessage.addListener((message) => {
  if (message.type === 'STATE_UPDATE') {
    updateUI(message.state);
  }
});

// Load initial state
getState().then(updateUI);

// Poll for updates
setInterval(() => {
  getState().then(updateUI);
}, 5000);
