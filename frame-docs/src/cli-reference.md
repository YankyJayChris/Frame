# CLI Reference

The `frame` CLI is the primary interface for creating, building, testing, deploying, and managing Frame projects.

## Commands

| Command | Description |
|---------|-------------|
| `frame start <name>` | Create new Frame project |
| `frame start <name> --arch clean` | Create with Clean Architecture scaffold |
| `frame start <name> --arch mvc` | Create with MVC architecture scaffold |
| `frame build` | Compile .fr files, check for errors |
| `frame build --watch` | Rebuild on file changes |
| `frame build --strict` | Treat warnings as errors |
| `frame deploy ios` | Deploy to iOS simulator (Xcode required) |
| `frame deploy android` | Deploy to Android emulator/device (SDK required) |
| `frame test` | Run all `.test.fr` test suites |
| `frame test --filter NAME` | Run tests matching NAME |
| `frame test --coverage` | Report test coverage |
| `frame preview` | Start hot-reload dev server (port 9001) |
| `frame lint` | Static analysis for style and correctness |
| `frame lint --rules FR001,FR010` | Run specific rules only |
| `frame lint --skip FR042` | Skip specific rules |
| `frame check` | Verify development environment |
| `frame check --fix` | Auto-install missing tools |
| `frame icon add <path>` | Register an SVG icon |
| `frame icon add <path> --name <name>` | Register with custom name |
| `frame icon load-bundle <path>` | Load icons from a .frameicons bundle file |
| `frame icon list` | List all registered icons |
| `frame icon generate` | Generate platform icon assets (PDF/XML) |
| `frame icon generate --target ios` | Generate only iOS assets |
| `frame icon generate --target android` | Generate only Android assets |
| `frame plugin add <name>` | Install a plugin from the Frame Plugin Registry |
| `frame plugin add @user/repo` | Install from GitHub |
| `frame plugin add @user/repo@v1.2.3` | Install specific version from GitHub |
| `frame plugin remove <name>` | Remove a plugin |
| `frame plugin list` | List installed plugins |
| `frame plugin create <name>` | Scaffold a new plugin |
| `frame init-examples` | Regenerate example projects |
| `frame lsp` | Start the Language Server Protocol server (used by VS Code extension) |
| `frame lsp --workspace-root <dir>` | Start LSP server with explicit project root |

## frame.config.json

The project configuration file lives at the root of every Frame project. All fields are optional unless noted.

```json
{
  "name": "myApp",
  "bundle_id": "com.example.myapp",
  "version": "1.0.0",
  "build_number": "1",
  "render_mode": "native",
  "min_android_sdk": 24,
  "min_ios": "16.0",
  "paths": {
    "@": "./src"
  },
  "plugins": {
    "frame_camera": "0.1.0",
    "frame_storage": "0.1.0",
    "frame_connectivity": "0.1.0"
  }
}
```

### Field Reference

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | — | Project name, used as the app display name |
| `bundle_id` | String | Yes | — | Unique app identifier (reverse-domain, e.g. `com.example.myapp`) |
| `version` | String | No | `"1.0.0"` | Semantic version string |
| `build_number` | String/Int | No | `"1"` | Incremental build number for app stores |
| `render_mode` | String | No | `"native"` | Rendering engine: `native` (default) or `experimental_skia` |
| `min_android_sdk` | Int | No | `24` | Minimum Android API level |
| `min_ios` | String | No | `"16.0"` | Minimum iOS version |
| `paths` | Object | No | `{ "@": "./src" }` | Path alias mapping — enables `@/` prefixed imports |
| `plugins` | Object | No | `{}` | Plugin dependencies with version constraints |

### `paths` field

The `paths` field enables the `@/` path alias. When set to `"./src"`, you can write:

```fr
import { MyComponent } "@/components/MyComponent"
```

This resolves to `<project_root>/src/components/MyComponent.fr`. The alias root is resolved relative to the project root (the directory containing `frame.config.json`).

### `plugins` field

Plugin dependencies are specified as key-value pairs where the key is the plugin name and the value is a semver version constraint:

```json
{
  "plugins": {
    "frame_camera": "0.1.0",
    "frame_storage": "^0.1.0",
    "frame_connectivity": "~0.1.0"
  }
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `FRAME_PATH` | Path to the Frame CLI binary (used by VS Code extension) |
| `FRAME_WORKSPACE_ROOT` | Override workspace root path |

## Build Command Details

### `frame build`

The `frame build` command is the main entry point for compiling your Frame project. It performs these steps:

1. **Parse** — Reads all `.fr` files from `src/` and validates syntax
2. **Resolve** — Resolves imports and checks for circular dependencies
3. **Type Check** — Validates prop types, function signatures, and async/await usage
4. **Generate Icons** — Creates platform-specific app icons (see [App Icons](app-icons.md))
5. **Generate Code** — Outputs platform-specific code for Android and iOS
6. **Cache** — Updates incremental build cache for faster rebuilds

### Icon Generation During Build

As part of the build process, Frame automatically:

1. Reads your app icon source (from `frame.config.json: icon` or uses default)
2. Validates the icon format (SVG, PNG, or JPEG)
3. Generates iOS app icons at 12 required sizes
4. Generates Android app icons at 6 density scales
5. Creates platform metadata (Contents.json for iOS, colors.xml for Android)

For detailed icon configuration, see [App Icons](app-icons.md).

### Watch Mode

Use `--watch` to rebuild automatically when files change:

```bash
frame build --watch
```

This is useful during development — your code recompiles instantly as you edit.

### Strict Mode

Use `--strict` to treat warnings as errors:

```bash
frame build --strict
```

This ensures code quality by catching potential issues early.

### Output

Build artifacts are written to:
- Android: `build/android/`
- iOS: `build/ios/`

After a successful build, you can deploy:

```bash
frame deploy android
frame deploy ios
```
