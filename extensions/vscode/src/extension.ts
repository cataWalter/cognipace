import * as vscode from 'vscode';

// State
let cognitiveState: any = null;
let statusBar: vscode.StatusBarItem;
let pollInterval: NodeJS.Timer | null = null;
let focusMode: boolean = false;

// State emoji mapping
const STATE_EMOJIS: Record<string, string> = {
  deep_focus: '🟢',
  flow_building: '🟡',
  fragmented: '🟠',
  fatigued: '🔴',
  overloaded: '⚫',
  recovering: '🔵',
};

export function activate(context: vscode.ExtensionContext) {
  console.log('CogniPace extension activated');

  // Create status bar item
  statusBar = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBar.command = 'cognipace.showStatus';
  statusBar.show();
  context.subscriptions.push(statusBar);

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('cognipace.showStatus', showStatus),
    vscode.commands.registerCommand('cognipace.toggleFocusMode', toggleFocusMode),
    vscode.commands.registerCommand('cognipace.takeBreak', takeBreak)
  );

  // Start polling
  startPolling();

  // Track typing for cognitive load analysis
  setupTypingTracker();
}

export function deactivate() {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
}

// Poll CogniPace API for state updates
function startPolling() {
  const config = vscode.workspace.getConfiguration('cognipace');
  const enabled = config.get<boolean>('enabled', true);
  
  if (!enabled) {
    statusBar.text = '🧠 Disabled';
    return;
  }

  updateState();
  
  pollInterval = setInterval(updateState, 5000);
}

async function updateState() {
  const config = vscode.workspace.getConfiguration('cognipace');
  const apiUrl = config.get<string>('apiUrl', 'http://localhost:8080');

  try {
    const response = await fetch(`${apiUrl}/api/state`, {
      signal: AbortSignal.timeout(2000)
    });

    if (response.ok) {
      cognitiveState = await response.json();
      updateStatusBar();
      applyAdaptations();
    }
  } catch {
    statusBar.text = '🧠 ⚪';
    statusBar.tooltip = 'CogniPace: Not connected';
  }
}

function updateStatusBar() {
  if (!cognitiveState) {
    statusBar.text = '🧠 ⚪';
    statusBar.tooltip = 'CogniPace: Waiting for data';
    return;
  }

  const emoji = STATE_EMOJIS[cognitiveState.attention_state] || '⚪';
  const loadPercent = Math.round(cognitiveState.load_score * 100);
  
  statusBar.text = `🧠 ${emoji} ${loadPercent}%`;
  statusBar.tooltip = `CogniPace: ${formatStateName(cognitiveState.attention_state)}\nLoad: ${loadPercent}%`;
  
  if (cognitiveState.load_score > 0.7) {
    statusBar.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
  } else if (cognitiveState.load_score > 0.5) {
    statusBar.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
  } else {
    statusBar.backgroundColor = undefined;
  }
}

function formatStateName(name: string): string {
  return name.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

function applyAdaptations() {
  if (!cognitiveState) return;

  const state = cognitiveState.attention_state;
  const config = vscode.workspace.getConfiguration('cognipace');
  const blockDistractions = config.get<boolean>('focusMode.blockDistractions', true);

  if (state === 'deep_focus' && blockDistractions) {
    vscode.workspace.getConfiguration('notifications').update('showNotifications', 'none');
  } else if (state === 'overloaded') {
    vscode.workspace.getConfiguration('workbench').update('editor.minimap.enabled', false);
    vscode.workspace.getConfiguration('workbench').update('activityBar.visible', false);
  } else {
    vscode.workspace.getConfiguration('notifications').update('showNotifications', 'all');
  }

  if (state === 'fatigued' || state === 'overloaded') {
    showBreakReminder();
  }
}

function showBreakReminder() {
  vscode.window.showInformationMessage(
    '🧠 CogniPace: Your cognitive load is high. Consider taking a break!',
    'Take 5 min break',
    'Dismiss'
  ).then(selection => {
    if (selection === 'Take 5 min break') {
      takeBreak();
    }
  });
}

async function showStatus() {
  if (!cognitiveState) {
    vscode.window.showInformationMessage('CogniPace: Not connected to CogniPace service');
    return;
  }

  const emoji = STATE_EMOJIS[cognitiveState.attention_state] || '⚪';
  const loadPercent = Math.round(cognitiveState.load_score * 100);

  const message = `🧠 CogniPace Status\n\n` +
    `State: ${emoji} ${formatStateName(cognitiveState.attention_state)}\n` +
    `Load: ${loadPercent}%\n` +
    `Focus Mode: ${focusMode ? 'ON' : 'OFF'}\n\n` +
    `Intrinsic Load: ${Math.round(cognitiveState.intrinsic_load * 100)}%\n` +
    `Extraneous Load: ${Math.round(cognitiveState.extraneous_load * 100)}%\n` +
    `Physiological Load: ${Math.round(cognitiveState.physiological_load * 100)}%`;

  vscode.window.showInformationMessage(message);
}

async function toggleFocusMode() {
  focusMode = !focusMode;
  
  if (focusMode) {
    vscode.window.showInformationMessage('🧠 Focus Mode enabled. Distractions minimized.');
    vscode.workspace.getConfiguration('workbench').update('editor.minimap.enabled', false);
    vscode.workspace.getConfiguration('workbench').update('statusBar.visible', false);
    vscode.workspace.getConfiguration('notifications').update('showNotifications', 'none');
  } else {
    vscode.window.showInformationMessage('🧠 Focus Mode disabled.');
    vscode.workspace.getConfiguration('workbench').update('editor.minimap.enabled', true);
    vscode.workspace.getConfiguration('workbench').update('statusBar.visible', true);
  }
}

async function takeBreak() {
  const breakActivities = [
    '🚶 Take a short walk',
    '🧘 Do some stretches',
    '🫁 Practice deep breathing',
    '💧 Drink some water',
    '👁️ Rest your eyes (20-20-20 rule)'
  ];
  
  const activity = breakActivities[Math.floor(Math.random() * breakActivities.length)];
  
  vscode.window.showInformationMessage(
    `🧠 Time for a break! Suggested activity: ${activity}`,
    'Start 5 min timer',
    'Dismiss'
  ).then(selection => {
    if (selection === 'Start 5 min timer') {
      startBreakTimer();
    }
  });
}

function startBreakTimer() {
  const breakDuration = 5 * 60 * 1000;
  const startTime = Date.now();
  
  vscode.window.withProgress({
    location: vscode.ProgressLocation.Notification,
    title: '🧠 Break in progress',
    cancellable: true
  }, async (progress, token) => {
    return new Promise((resolve) => {
      const interval = setInterval(() => {
        const elapsed = Date.now() - startTime;
        const remaining = Math.max(0, breakDuration - elapsed);
        const remainingMin = Math.floor(remaining / 60000);
        const remainingSec = Math.floor((remaining % 60000) / 1000);
        
        progress.report({
          message: `${remainingMin}:${remainingSec.toString().padStart(2, '0')} remaining`
        });
        
        if (remaining <= 0) {
          clearInterval(interval);
          resolve();
        }
      }, 1000);
      
      token.onCancellationRequested(() => {
        clearInterval(interval);
        resolve();
      });
    });
  });
}

function setupTypingTracker() {
  let lastTypeTime = Date.now();
  let typeTimes: number[] = [];
  
  vscode.workspace.onDidChangeTextDocument((e) => {
    const now = Date.now();
    const timeSinceLastType = now - lastTypeTime;
    
    if (timeSinceLastType < 1000) {
      typeTimes.push(timeSinceLastType);
    } else {
      if (typeTimes.length > 0) {
        const avgTime = typeTimes.reduce((a, b) => a + b, 0) / typeTimes.length;
        (global as any).cognipaceTypingSpeed = avgTime;
      }
      typeTimes = [];
    }
    
    lastTypeTime = now;
  });
}
