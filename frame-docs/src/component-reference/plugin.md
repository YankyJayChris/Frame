# Plugin Component

The `plugin` component bridges to native plugin functionality. It is a special built-in component that does not accept children or style props — it's purely a call-site bridge that invokes a method on a registered native plugin.

## Syntax

```fr
plugin: {
    name: "frame_camera"
    method: "captureImage"
}
```

## Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `name` | String | **Yes** | — | Plugin name (matches the plugin identifier) |
| `method` | String | **Yes** | — | Method name to call on the plugin |

| Styles | Events | Children |
|--------|--------|----------|
| — | — | No |

## Usage

The `plugin` component can be placed anywhere in the component tree. It acts as a transparent bridge — it invokes the specified method on the native plugin at the appropriate lifecycle point.

```fr
// Track an analytics event
plugin: { name: "analytics"  method: "trackEvent" }

// Capture a photo
plugin: { name: "frame-camera"  method: "captureImage" }

// Save a file
plugin: { name: "frame-storage"  method: "saveFile" }

// Check network connectivity
plugin: { name: "frame-connectivity"  method: "isOnline" }
```

## How Plugins Work

1. Plugins are installed via `frame plugin add <name>` and appear in `frame_modules/`
2. Each plugin provides a `plugin.json` manifest with its name, version, methods, and required permissions
3. The `plugin:` component resolves the method call to the native implementation at compile time
4. Plugin methods can also be called directly in Frame functions using `wait:pluginName.methodName()` syntax:

```fr
import { capture } "frame-camera"

fn handleCapture: async () => {
    :var photo = wait:capture("jpg", 0.9, "camera")
    if photo != null {
        UserStore.photo = photo
    }
}
```

For more details on installing, creating, and managing plugins, see the [Plugin System](../plugin-system.md) documentation.
