# Plugin System

Frame plugins extend the framework with native functionality, custom components, and platform permissions.

## Architecture

Plugins are native code modules that communicate with Frame through a stable ABI. Each plugin provides:

- **Frame DSL bindings** — functions and components callable from `.fr` files
- **Native implementations** — Kotlin code for Android, Swift code for iOS
- **Plugin manifest** — `plugin.json` with name, version, methods, and permissions

```
┌─────────────────────────────────────────────┐
│  Frame Compiler                              │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Plugin       │  │  Native Codegen      │  │
│  │  Registry     │  │  (Kotlin/Swift)     │  │
│  └──────┬───────┘  └──────────┬───────────┘  │
└─────────┼────────────────────┼──────────────┘
          │                    │
┌─────────▼────────────────────▼──────────────┐
│  Plugin Module                                │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │  plugin.json  │  │  Native Implement.   │  │
│  │  (manifest)   │  │  (kotlin/ iOS.swift) │  │
│  └──────────────┘  └──────────────────────┘  │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │  index.fr     │  │  Assets/Icons       │  │
│  └──────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────┘
```

## Plugin Discovery

Plugins are discovered from two sources:

1. **`frame.config.json`** — the `plugins` field lists dependencies with version constraints
2. **`frame_modules/`** — installed plugin packages live in this directory

```json
{
  "plugins": {
    "frame_camera": "0.1.0",
    "frame_storage": "0.1.0",
    "frame_connectivity": "0.1.0"
  }
}
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `frame plugin add <name>` | Install a plugin from the Frame Plugin Registry |
| `frame plugin add @user/repo` | Install from GitHub |
| `frame plugin add @user/repo@v1.2.3` | Install specific version from GitHub |
| `frame plugin list` | List installed plugins |
| `frame plugin remove <name>` | Remove a plugin |
| `frame plugin create <name>` | Scaffold a new plugin |

## Using Plugins in .fr Files

### The `plugin:` Component

```fr
plugin: { name: "frame_camera"  method: "captureImage" }
plugin: { name: "analytics"  method: "trackEvent" }
plugin: { name: "frame-storage"  method: "saveFile" }
```

### The `plugin:` Call Statement

For async plugin methods, use the `wait:` prefix with a function import:

```fr
import { capture } "frame-camera"

fn handleCapture: async () => {
    :var photo = wait:capture("jpg", 0.9, "camera")
    if photo != null {
        UserStore.photo = photo
    }
}
```

## Permission Management

Plugins automatically declare required permissions in their `plugin.json`. The Frame compiler injects these into the platform build files:

| Plugin | iOS Permissions | Android Permissions |
|--------|-----------------|---------------------|
| `frame_camera` | `NSCameraUsageDescription` | `android.permission.CAMERA` |
| `frame_storage` | — | `android.permission.WRITE_EXTERNAL_STORAGE` |
| `frame_connectivity` | — | `android.permission.ACCESS_NETWORK_STATE` |

Create a Plugin

```bash
frame plugin create my-plugin
```

A plugin directory contains:

```
my-plugin/
├── plugin.json               # Manifest (name, version, permissions)
├── src/
│   └── index.fr              # Frame components/functions
├── android/
│   └── MyPlugin.kt           # Kotlin native implementation
├── ios/
│   └── MyPlugin.swift        # Swift native implementation
└── assets/
    └── icons/                # Plugin-specific icons
```

### plugin.json Manifest

```json
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "My custom Frame plugin",
  "methods": {
    "doSomething": {
      "async": true,
      "params": [{ "name": "arg", "type": "string" }],
      "return": "string"
    }
  },
  "permissions": {
    "ios": ["NSCameraUsageDescription"],
    "android": ["android.permission.CAMERA"]
  }
}
```

## Available Plugins

### frame_camera

Camera capture plugin — take photos from device camera.

```fr
import { capture } "frame-camera"

fn handleCapture: async () => {
    :var photo = wait:capture("jpg", 0.9, "camera")
    if photo != null {
        UserStore.photo = photo
    }
}
```

### frame_storage

Local file storage plugin — save, read, and manage files.

| Function | Description |
|----------|-------------|
| `wait:saveFile(filename, data)` | Write data to file |
| `wait:readFile(filename)` | Read file contents |
| `wait:deleteFile(filename)` | Delete a file |
| `wait:listFiles()` | List all files |
| `wait:fileExists(filename)` | Check if file exists |

### frame_connectivity

Network connectivity monitoring plugin.

| Function | Description |
|----------|-------------|
| `wait:isOnline(type)` | Check connectivity (`"any"`, `"wifi"`, `"cellular"`) |
| `wait:onNetworkChange()` | Subscribe to network state changes |
