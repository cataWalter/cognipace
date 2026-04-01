# CogniPace API Documentation

## REST API

CogniPace exposes a REST API for integration with extensions and third-party tools.

### Base URL

```
http://localhost:8080/api
```

### Endpoints

#### Get Current Cognitive State

```
GET /api/state
```

**Response:**
```json
{
  "id": "uuid",
  "timestamp": "2026-04-01T14:30:00Z",
  "attention_state": "deep_focus",
  "load_score": 0.25,
  "intrinsic_load": 0.15,
  "extraneous_load": 0.20,
  "physiological_load": 0.10,
  "forecast": {
    "predicted_load": 0.30,
    "confidence": 0.75,
    "horizon_seconds": 900,
    "overload_probability": 0.05
  }
}
```

#### Get Configuration

```
GET /api/config
```

**Response:**
```json
{
  "sensors": {
    "eye_tracking": { "enabled": true, "sample_rate": 30 },
    "keystroke": { "enabled": true },
    "app_usage": { "enabled": true }
  },
  "adaptations": {
    "notifications": { "enabled": true, "mode": "adaptive" },
    "ui_density": { "enabled": true },
    "breaks": { "enabled": true, "interval_minutes": 90 }
  }
}
```

#### Get Statistics

```
GET /api/stats
```

**Response:**
```json
{
  "feature_count": 1250,
  "state_count": 450,
  "adaptation_count": 120,
  "retention_days": 7
}
```

#### Purge Data

```
POST /api/purge
```

**Response:** `200 OK`

## Tauri Commands

For the desktop dashboard, use these Tauri invoke calls:

```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Get current cognitive state
const state = await invoke('get_cognitive_state');

// Get configuration
const config = await invoke('get_config');

// Get statistics
const stats = await invoke('get_stats');

// Purge all data
await invoke('purge_data');
```

## Browser Extension API

The browser extension exposes messages for content scripts:

```javascript
// Get current state
chrome.runtime.sendMessage({ type: 'GET_STATE' }, (state) => {
  console.log(state);
});

// Request UI reduction
chrome.runtime.sendMessage({ type: 'REDUCE_UI' });

// Request page summarization
chrome.runtime.sendMessage({ type: 'SUMMARIZE' });
```

## VS Code Extension Commands

| Command | Description |
|---------|-------------|
| `cognipace.showStatus` | Show current cognitive status |
| `cognipace.toggleFocusMode` | Toggle focus mode |
| `cognipace.takeBreak` | Start a break timer |

## Configuration

See [default.yaml](../configs/default.yaml) for all configuration options.
