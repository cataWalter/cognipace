# Privacy & Ethics Design

## Core Principles

1. **Local-First Processing**: All sensor data processing happens on-device
2. **Zero Telemetry**: No data leaves the device without explicit user action
3. **Ephemeral Raw Data**: Raw sensor data is never stored to disk
4. **Transparent Decisions**: Every adaptation is logged with reasoning
5. **User Control**: Manual override available at all times

## Data Lifecycle

### Sensor Data Flow

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Capture    │───▶│   Process    │───▶│   Extract    │
│              │    │              │    │              │
│ Raw frames   │    │ In-memory    │    │ Feature      │
│ Keystrokes   │    │ processing   │    │ vectors only │
│ App events   │    │              │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
     (seconds)            (ms)               (stored)
```

### What We Store

| Data Type | Stored? | Retention | Purpose |
|-----------|---------|-----------|---------|
| Raw webcam frames | ❌ Never | N/A | Eye tracking input |
| Raw keystrokes | ❌ Never | N/A | Timing analysis |
| Feature vectors | ✅ Yes | Configurable (default: 7 days) | Cognitive load calculation |
| Cognitive state history | ✅ Yes | Configurable (default: 30 days) | Trend analysis |
| Adaptation decisions | ✅ Yes | Configurable (default: 30 days) | Audit trail |
| User feedback | ✅ Yes | Until deleted | Model personalization |

### What We NEVER Store

- Raw video or audio recordings
- Keystroke content (only timing metadata)
- Screen content or screenshots
- Personal files or documents
- Network traffic or browsing history

## Sensor Permissions

Each sensor requires explicit user consent:

```yaml
# Example consent prompt
┌─────────────────────────────────────────────────────┐
│  CogniPace requests access to:                      │
│                                                     │
│  📷 Webcam                                          │
│  Purpose: Eye tracking for focus detection          │
│  Data stored: No raw frames, only gaze coordinates  │
│  Processing: 100% local, no network calls           │
│                                                     │
│  [Allow] [Deny] [Learn More]                        │
└─────────────────────────────────────────────────────┘
```

## Encryption

- **Database**: SQLite with SQLCipher encryption (AES-256)
- **Configuration**: Encrypted at rest
- **Model weights**: Integrity verified via checksums

## Audit Trail

Every adaptation decision is logged:

```json
{
  "timestamp": "2026-04-01T14:30:00Z",
  "cognitive_state": "fragmented",
  "load_score": 0.78,
  "adaptation": "batch_notifications",
  "reason": "High app switching rate (12/min) + gaze variance detected",
  "user_override": false
}
```

## User Rights

### Data Export
```bash
# Export all stored data
cognipace-cli export --format json --output ~/cognipace-data.json
```

### Data Deletion
```bash
# Delete all stored data
cognipace-cli purge --confirm
```

### Sensor Control
```bash
# Disable specific sensors
cognipace-cli sensors disable eye_tracking

# View active sensors
cognipace-cli sensors status
```

## Ethical Guardrails

### What CogniPace Will NEVER Do

1. **Manipulate behavior**: No dark patterns or engagement optimization
2. **Hide information**: All adaptations are transparent and reversible
3. **Sell data**: No data collection, period
4. **Require cloud**: Fully functional offline
5. **Force adaptations**: User always has final say

### What CogniPace WILL Do

1. **Respect autonomy**: User preferences override AI suggestions
2. **Explain decisions**: Every action comes with reasoning
3. **Protect privacy**: Minimal data collection, maximum security
4. **Stay transparent**: Open source code, auditable by anyone
5. **Serve wellbeing**: Optimized for user health, not engagement

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|-----------|
| Local data theft | SQLCipher encryption |
| Sensor hijacking | OS-level permission prompts |
| Model poisoning | Signed model weights |
| Privilege escalation | Minimal required permissions |
| Side-channel attacks | Constant-time operations where applicable |

### Permission Requirements

| Platform | Required Permissions | Purpose |
|----------|---------------------|---------|
| Linux | evdev access, DBus | Keystroke timing, notifications |
| macOS | Accessibility, Camera | Input monitoring, eye tracking |
| Windows | UI Automation | App usage monitoring |

## Compliance

CogniPace is designed to comply with:
- GDPR (EU General Data Protection Regulation)
- CCPA (California Consumer Privacy Act)
- HIPAA (when used in healthcare contexts)

## Reporting Privacy Issues

If you discover a privacy vulnerability, please report it responsibly:
- Email: security@cognipace.dev
- PGP Key: Available at https://cognipace.dev/security
