# iOS Build Guide

## Prerequisites

- Xcode 15+ (macOS only)
- Apple Developer account (for device testing)
- iOS Simulator or physical device
- XcodeGen: `brew install xcodegen`

## Project Structure

```
src-tauri/
├── Info.ios.plist          # Tauri iOS plist (local network permissions)
├── ios/
│   ├── project.yml         # XcodeGen project definition
│   └── Info.plist          # iOS app Info.plist (copied from above)
```

## Generate Xcode Project

```bash
cd src-tauri/ios
xcodegen generate
open Lumin.xcodeproj
```

## Build for Simulator

```bash
xcodebuild -project Lumin.xcodeproj \
  -scheme Lumin \
  -configuration Debug \
  -destination 'platform=iOS Simulator,name=iPhone 15' \
  build
```

## Build for Device

1. Set `DEVELOPMENT_TEAM` in `ios/project.yml`
2. Set `CODE_SIGN_IDENTITY` to your signing certificate
3. Set `PROVISIONING_PROFILE_SPECIFIER` to your provisioning profile
4. Build:

```bash
xcodebuild -project Lumin.xcodeproj \
  -scheme Lumin \
  -configuration Release \
  -destination 'generic/platform=iOS' \
  build
```

## Local Network Permissions

The app requires local network access for Bonjour service discovery:

- **NSLocalNetworkUsageDescription**: "Lumin は教室内の生徒と通信するためにローカルネットワークを使用します。"
- **NSBonjourServices**: `_lumin-class._tcp`

These are configured in both `src-tauri/Info.ios.plist` and `src-tauri/ios/Info.plist`.

## Tauri CLI Note

The `tauri ios init` command requires the Tauri CLI with iOS support. If unavailable, use XcodeGen as described above. The `project.yml` in `src-tauri/ios/` provides the same project structure.
