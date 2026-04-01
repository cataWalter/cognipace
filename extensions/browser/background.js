// CogniPace Browser Extension - Background Service Worker

const COGNIPACE_API_URL = 'http://localhost:8080';

// State
let cognitiveState = null;
let config = null;
let updateInterval = null;

// State emoji mapping
const STATE_EMOJIS = {
  deep_focus: '🟢',
  flow_building: '🟡',
  fragmented: '🟠',
  fatigued: '🔴',
  overloaded: '⚫',
  recovering: '🔵',
};

// Initialize extension
async function init() {
  console.log('CogniPace: Initializing...');
  
  // Load saved config
  const result = await chrome.storage.local.get(['config', 'cognitiveState']);
  if (result.config) {
    config = result.config;
  }
  if (result.cognitiveState) {
    cognitiveState = result.cognitiveState;
    updateBadge(cognitiveState);
  }
  
  // Start polling for state updates
  startPolling();
  
  // Listen for messages from content scripts
  chrome.runtime.onMessage.addListener(handleMessage);
  
  console.log('CogniPace: Initialized');
}

// Poll CogniPace API for state updates
function startPolling() {
  if (updateInterval) {
    clearInterval(updateInterval);
  }
  
  updateInterval = setInterval(async () => {
    try {
      const response = await fetch(`${COGNIPACE_API_URL}/api/state`, {
        method: 'GET',
        signal: AbortSignal.timeout(2000)
      });
      
      if (response.ok) {
        const newState = await response.json();
        cognitiveState = newState;
        await chrome.storage.local.set({ cognitiveState: newState });
        updateBadge(newState);
        
        // Notify content scripts of state change
        chrome.tabs.query({}, (tabs) => {
          tabs.forEach(tab => {
            chrome.tabs.sendMessage(tab.id, {
              type: 'STATE_UPDATE',
              state: newState
            }).catch(() => {});
          });
        });
      }
    } catch (e) {
      // API not available - extension works standalone
    }
  }, 5000); // Poll every 5 seconds
}

// Update extension badge with cognitive state
function updateBadge(state) {
  if (!state || !state.attention_state) {
    chrome.action.setBadgeText({ text: '' });
    return;
  }
  
  const emoji = STATE_EMOJIS[state.attention_state] || '⚪';
  const loadPercent = Math.round(state.load_score * 100);
  
  chrome.action.setBadgeText({ text: `${loadPercent}%` });
  chrome.action.setBadgeBackgroundColor({ 
    color: getLoadColor(state.load_score) 
  });
  chrome.action.setTitle({ 
    title: `CogniPace: ${formatStateName(state.attention_state)} (${loadPercent}% load)` 
  });
}

// Get color based on load score
function getLoadColor(load) {
  if (load < 0.3) return '#4caf50';
  if (load < 0.5) return '#ffeb3b';
  if (load < 0.7) return '#ff9800';
  if (load < 0.9) return '#f44336';
  return '#9e9e9e';
}

// Format state name
function formatStateName(name) {
  return name.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

// Handle messages from popup and content scripts
function handleMessage(message, sender, sendResponse) {
  switch (message.type) {
    case 'GET_STATE':
      sendResponse(cognitiveState);
      break;
      
    case 'GET_CONFIG':
      sendResponse(config);
      break;
      
    case 'SUMMARIZE_PAGE':
      // Extract page content for summarization
      chrome.tabs.sendMessage(sender.tab.id, { type: 'GET_PAGE_CONTENT' }, (content) => {
        sendResponse({ content });
      });
      return true; // Keep message channel open
      
    case 'REDUCE_UI':
      // Apply UI reduction to current tab
      chrome.tabs.sendMessage(sender.tab.id, { type: 'REDUCE_UI' });
      sendResponse({ success: true });
      break;
  }
}

// Handle tab updates for app usage tracking
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    // Track page visits for cognitive load analysis
    chrome.storage.local.get(['pageVisits'], (result) => {
      const visits = result.pageVisits || [];
      visits.push({
        url: tab.url,
        title: tab.title,
        timestamp: Date.now()
      });
      
      // Keep only last 100 visits
      if (visits.length > 100) {
        visits.splice(0, visits.length - 100);
      }
      
      chrome.storage.local.set({ pageVisits: visits });
    });
  }
});

// Initialize on startup
init();
