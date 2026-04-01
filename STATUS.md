# Project Status

## Current Status: Experimental 🧪

CogniPace is in early development. The core architecture is defined and initial implementations are in progress.

## Implementation Status

### Core Engine
- [x] Type definitions
- [x] Error handling
- [x] Cognitive load model (rule-based)
- [x] Attention state classifier
- [x] Database storage
- [ ] ONNX model integration

### Sensors
- [x] Sensor trait abstraction
- [x] Eye tracker (stub)
- [x] Keystroke dynamics (stub)
- [x] App usage monitor (stub)
- [ ] MediaPipe integration
- [ ] OS-level input hooks

### Adaptations
- [x] Notification router
- [x] UI density controller
- [x] App availability manager
- [x] Break scheduler
- [ ] Content summarizer

### Interfaces
- [x] CLI with TUI
- [x] Tauri dashboard (stub)
- [x] Browser extension
- [x] VS Code extension

### ML Pipeline
- [x] Model architecture (PyTorch)
- [x] Training script
- [ ] Dataset collection
- [ ] Model evaluation

## Known Limitations

1. **Sensor implementations are stubs** - Real sensor implementations require OS-specific integrations that are planned but not yet implemented.

2. **No real ML model yet** - The current cognitive load calculation uses rule-based heuristics. ML model integration is planned.

3. **Privacy permissions** - Some features require OS permissions that need manual setup.

## Roadmap

See [README.md](README.md#roadmap) for the full development roadmap.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
