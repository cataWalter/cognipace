// CogniPace Browser Extension - Content Script

// UI Reduction styles
const UI_REDUCTION_STYLES = `
  .cognipace-reduced * { visibility: hidden !important; }
  .cognipace-reduced .cognipace-essential { visibility: visible !important; }
  .cognipace-reduced body { background: #1a1a2e !important; }
  .cognipace-reduced article, .cognipace-reduced main, 
  .cognipace-reduced .content, .cognipace-reduced #content { visibility: visible !important; }
`;

// Inject styles
const styleSheet = document.createElement('style');
styleSheet.textContent = UI_REDUCTION_STYLES;
document.head.appendChild(styleSheet);

// Listen for messages from background/popup
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  switch (message.type) {
    case 'STATE_UPDATE':
      handleStateUpdate(message.state);
      break;
      
    case 'REDUCE_UI':
      reduceUI();
      sendResponse({ success: true });
      break;
      
    case 'SUMMARIZE':
      summarizePage();
      sendResponse({ success: true });
      break;
      
    case 'GET_PAGE_CONTENT':
      sendResponse(extractPageContent());
      break;
  }
});

// Handle cognitive state updates
function handleStateUpdate(state) {
  if (!state) return;
  
  // Update page based on cognitive state
  switch (state.attention_state) {
    case 'deep_focus':
      // Hide distractions
      hideElements([
        '.sidebar', '.ads', '.recommended', '.trending',
        '[data-testid="sidebar"]', '.social-feed'
      ]);
      break;
      
    case 'fragmented':
    case 'fatigued':
      // Simplify page
      simplifyPage();
      break;
      
    case 'overloaded':
      // Emergency simplification
      emergencySimplify();
      break;
  }
  
  // Show break reminder if needed
  if (state.attention_state === 'fatigued' || state.attention_state === 'overloaded') {
    showBreakReminder(state);
  }
}

// Hide elements matching selectors
function hideElements(selectors) {
  selectors.forEach(selector => {
    document.querySelectorAll(selector).forEach(el => {
      el.style.opacity = '0.1';
      el.style.pointerEvents = 'none';
    });
  });
}

// Simplify page for fragmented/fatigued states
function simplifyPage() {
  // Increase font size
  document.body.style.fontSize = '18px';
  document.body.style.lineHeight = '1.8';
  
  // Reduce visual clutter
  document.querySelectorAll('img:not(.cognipace-essential)').forEach(img => {
    img.style.opacity = '0.5';
  });
  
  // Hide non-essential elements
  hideElements([
    '.ads', '.popup', '.modal', '.notification-bar',
    '.cookie-banner', '.newsletter-signup'
  ]);
}

// Emergency simplification for overload
function emergencySimplify() {
  document.body.classList.add('cognipace-reduced');
  
  // Mark essential content
  document.querySelectorAll('article, main, .content, #content').forEach(el => {
    el.classList.add('cognipace-essential');
  });
  
  // Show overlay with break suggestion
  showBreakReminder({
    attention_state: 'overloaded',
    load_score: 1.0
  });
}

// Show break reminder
function showBreakReminder(state) {
  // Check if already shown
  if (document.getElementById('cognipace-break-reminder')) return;
  
  const reminder = document.createElement('div');
  reminder.id = 'cognipace-break-reminder';
  reminder.style.cssText = `
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: #0f3460;
    color: #e8e8e8;
    padding: 16px 24px;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    z-index: 10000;
    font-family: system-ui, sans-serif;
    max-width: 300px;
  `;
  
  const activities = ['🚶 Take a walk', '🧘 Stretch', '🫁 Deep breathing', '💧 Hydrate'];
  const activity = activities[Math.floor(Math.random() * activities.length)];
  
  reminder.innerHTML = `
    <div style="font-size: 14px; font-weight: 600; margin-bottom: 8px;">
      🧠 CogniPace suggests a break
    </div>
    <div style="font-size: 12px; color: #a0a0a0; margin-bottom: 12px;">
      ${activity}
    </div>
    <button onclick="this.parentElement.remove()" style="
      background: #e94560;
      color: white;
      border: none;
      padding: 6px 12px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 12px;
    ">Dismiss</button>
  `;
  
  document.body.appendChild(reminder);
  
  // Auto-remove after 30 seconds
  setTimeout(() => {
    if (reminder.parentElement) {
      reminder.remove();
    }
  }, 30000);
}

// Extract page content for summarization
function extractPageContent() {
  // Get main content
  const mainContent = document.querySelector('article, main, .content, #content') 
    || document.body;
  
  return {
    title: document.title,
    url: window.location.href,
    text: mainContent?.innerText?.substring(0, 5000) || '',
    wordCount: mainContent?.innerText?.split(/\s+/).length || 0
  };
}

// Summarize page (placeholder - would call local LLM)
function summarizePage() {
  const content = extractPageContent();
  
  // Show loading indicator
  const indicator = document.createElement('div');
  indicator.id = 'cognipace-summary';
  indicator.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: #0f3460;
    color: #e8e8e8;
    padding: 16px;
    border-radius: 12px;
    max-width: 400px;
    max-height: 80vh;
    overflow-y: auto;
    z-index: 10000;
    font-family: system-ui, sans-serif;
  `;
  
  indicator.innerHTML = `
    <div style="font-weight: 600; margin-bottom: 8px;">📝 Page Summary</div>
    <div style="font-size: 12px; color: #a0a0a0;">
      Generating summary... (Local LLM integration required)
    </div>
    <div style="margin-top: 12px; padding-top: 12px; border-top: 1px solid #16213e;">
      <strong>Title:</strong> ${content.title}<br>
      <strong>Words:</strong> ${content.wordCount}
    </div>
    <button onclick="this.parentElement.remove()" style="
      margin-top: 12px;
      background: #e94560;
      color: white;
      border: none;
      padding: 6px 12px;
      border-radius: 6px;
      cursor: pointer;
    ">Close</button>
  `;
  
  document.body.appendChild(indicator);
}

// Reduce UI complexity
function reduceUI() {
  if (document.body.classList.contains('cognipace-reduced')) {
    // Restore
    document.body.classList.remove('cognipace-reduced');
    document.querySelectorAll('.cognipace-essential').forEach(el => {
      el.classList.remove('cognipace-essential');
    });
  } else {
    simplifyPage();
  }
}

// Track reading speed for cognitive load analysis
let lastScrollTime = Date.now();
let lastScrollPosition = window.scrollY;

window.addEventListener('scroll', () => {
  const now = Date.now();
  const timeDiff = (now - lastScrollTime) / 1000; // seconds
  const scrollDiff = Math.abs(window.scrollY - lastScrollPosition);
  
  if (timeDiff > 0 && scrollDiff > 0) {
    // Estimate reading speed (pixels per second)
    const readingSpeed = scrollDiff / timeDiff;
    
    // Store for analysis (would send to CogniPace core)
    window.cognipaceReadingSpeed = readingSpeed;
  }
  
  lastScrollTime = now;
  lastScrollPosition = window.scrollY;
});

console.log('CogniPace content script loaded');
