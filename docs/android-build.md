# Android Build Guide

## Prerequisites
- Android Studio (latest stable)
- Android SDK API 24+ (minimum)
- Java 17+

## Manifest Location
`src-tauri/AndroidManifest.xml` — Tauri 2 uses this as the primary Android manifest.

## Local Network Permissions
The manifest declares these permissions for peer discovery:

| Permission | Purpose |
|---|---|
| `INTERNET` | HTTP server, WebSocket connections |
| `ACCESS_NETWORK_STATE` | Check network availability before discovery |
| `ACCESS_WIFI_STATE` | Read Wi-Fi status for connectivity checks |
| `CHANGE_WIFI_MULTICAST_STATE` | Required for mDNS/Bonjour peer discovery |

## Build
1. Open in Android Studio: `File > Open > src-tauri`
2. Sync Gradle: `File > Sync Project with Gradle Files`
3. Build APK: `Build > Build APK` or command line:
   ```bash
   cd src-tauri && ./gradlew assembleDebug
   ```
4. APK output: `app/build/outputs/apk/debug/app-debug.apk`

## Notes
- Tauri 2 generates the final merged manifest at build time
- `src-tauri/AndroidManifest.xml` provides base permissions merged into the generated manifest
- Do not commit keystore files or credentials
