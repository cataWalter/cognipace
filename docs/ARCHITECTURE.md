# CogniPace Architecture

## System Overview

CogniPace is built as a modular, layered architecture designed for performance, privacy, and extensibility.

## Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   CLI       │  │  Dashboard  │  │  Browser Extension  │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   ADAPTATION LAYER                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │Notification │  │    UI       │  │   App Availability  │ │
│  │   Router    │  │  Density    │  │     Manager         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│  ┌─────────────┐  ┌─────────────┐                          │
│  │  Content    │  │   Break     │                          │
│  │ Summarizer  │  │ Scheduler   │                          │
│  └─────────────┘  └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   COGNITIVE ENGINE                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Feature    │  │   State     │  │    Prediction       │ │
│  │ Extraction  │  │ Classifier  │  │      Engine         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    SENSING LAYER                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │    Eye      │  │  Keystroke  │  │    App Usage        │ │
│  │  Tracking   │  │  Dynamics   │  │     Monitor         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Calendar   │  │  Wearables  │  │   Voice Stress      │ │
│  │  Context    │  │   (BLE)     │  │   Detection         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Crate Dependencies

```
cognipace-dashboard ──┐
                      ├──► cognipace-adaptations ──► cognipace-core
cognipace-cli     ────┘                                    ▲
                                                           │
                                              cognipace-sensors
```

## Data Flow

1. **Sensors** collect raw data (eye position, keystroke timing, app switches)
2. **Feature Extraction** converts raw data into anonymized feature vectors
3. **Cognitive Engine** classifies current state and predicts future load
4. **Adaptations** modify system behavior based on predictions
5. **UI** displays state and allows user override

## Privacy Architecture

All data processing happens locally:

```
Raw Sensor Data ──▶ Feature Extraction ──▶ Cognitive Model
     (RAM only)         (local)              (local)
         │                   │                   │
         ▼                   ▼                   ▼
    Immediately          Store only         No network
     discarded         feature vectors      calls needed
```

## Platform Support

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Eye Tracking | ✅ | ✅ | ✅ |
| Keystroke Dynamics | ✅ (evdev) | ✅ (CGEvent) | ✅ (Win32) |
| App Usage | ✅ (X11/Wayland) | ✅ | ✅ |
| Notifications | ✅ (DBus) | ✅ | ✅ |
| Calendar | ✅ | ✅ | ✅ |
| Wearables | ✅ (BLE) | ✅ (HealthKit) | ✅ |
