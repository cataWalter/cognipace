# Platform Support

## Linux

### Supported Features
- ✅ Eye Tracking (via MediaPipe)
- ✅ Keystroke Dynamics (via evdev)
- ✅ App Usage (via X11/Wayland)
- ✅ Notifications (via DBus)
- ✅ Calendar Integration
- ✅ Wearable Sync (via Bluetooth LE)

### Requirements
```bash
# Ubuntu/Debian
sudo apt install libevdev-dev libdbus-1-dev

# Fedora
sudo dnf install libevdev-devel dbus-devel
```

## macOS

### Supported Features
- ✅ Eye Tracking (via MediaPipe)
- ✅ Keystroke Dynamics (via CGEvent)
- ✅ App Usage (via Accessibility API)
- ✅ Notifications (via UserNotifications)
- ✅ Calendar (via EventKit)
- ✅ Wearable Sync (via HealthKit)

### Requirements
- macOS 12.0+
- Accessibility permissions required
- Camera permissions for eye tracking

## Windows

### Supported Features
- ✅ Eye Tracking (via MediaPipe)
- ✅ Keystroke Dynamics (via Win32 API)
- ✅ App Usage (via UI Automation)
- ✅ Notifications (via WinRT)
- ✅ Calendar (via UWP APIs)
- ✅ Wearable Sync (via Bluetooth)

### Requirements
- Windows 10+
- .NET Framework 4.8+
