# Getting Started

This guide will walk you through installing Frame, creating your first project, and running it on a device or simulator.

## Prerequisites

- **Rust** (1.70+). Install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Android SDK** (for Android builds) — Android Studio recommended
- **Xcode** (for iOS builds) — macOS only

## Installation

Install the Frame CLI via Cargo:

```bash
cargo install frame
```

This installs the `frame` binary along with the LSP server, compiler, and all tooling.

### Building from Source

Alternatively, clone the repository and build from source:

```bash
git clone https://github.com/frame-lang/frame.git
cd frame
cargo build --release
```

### PATH Setup

After installation, ensure the Frame binary is on your PATH:

```bash
# If installed via cargo
export PATH="$HOME/.cargo/bin:$PATH"

# If built from source
ln -sf "$(pwd)/target/release/frame" ~/.local/bin/frame
export PATH="$HOME/.local/bin:$PATH"
```

Verify the installation:

```bash
frame --version
```

## Creating a New Project

Scaffold a new Frame project with:

```bash
frame start myApp
cd myApp
```

This generates the following project structure:

```
myApp/
├── src/
│   ├── project.fr            # Entry point — :vars, :app, pages, top-level fns
│   ├── views/pages/          # MVC: page definitions
│   ├── views/components/     # MVC: reusable components
│   ├── models/               # MVC: :store state + :obj types
│   ├── controllers/          # MVC: business-logic functions
│   └── tests/                # *.test.fr test suites
├── assets/
│   ├── icons/                # SVG icons & .frameicons bundle files
│   ├── images/               # Image assets
│   └── fonts/                # TTF/OTF font files
├── frame_modules/            # Installed plugins
├── frame.config.json         # Bundle ID, version, plugins
└── frame-icons.json          # Registered icon manifest
```

### Configuration

The `frame.config.json` file controls your app's metadata:

```json
{
  "name": "myApp",
  "bundle_id": "com.example.myApp",
  "version": "1.0.0",
  "plugins": [],
  "icons": {
    "sets": ["default"]
  }
}
```

## Verifying Your Environment

Run the environment check to ensure all dependencies are in place:

```bash
frame check
```

This checks for:
- Required SDK paths (Android SDK, Xcode)
- Toolchain versions
- Project configuration validity

## Hello World

Open `src/project.fr` and replace its contents with:

```fr
:vars {
    primary: "#007BFF"
}

:app {
    on_launch: appInit
}

fn appInit: () => {
    log.info("Hello from Frame!")
}

page: {
    name: "Home"
    route: "/"
    styles: { width: 100%  height: 100% }
    children: [
        scaffold: {
            styles: { safe_area: true }
            children: [
                app_bar: { title: "Frame App" }
                column: {
                    styles: {
                        width: 100%
                        height: 100%
                        padding: 32
                        align: "center"
                        justify: "center"
                        gap: 16
                    }
                    children: [
                        text: {
                            content: "Hello, World!"
                            styles: {
                                font_size: 32sp
                                font_weight: "bold"
                                color: $primary
                            }
                        }
                        text: {
                            content: "Welcome to Frame"
                            styles: { font_size: 16sp  color: "#666" }
                        }
                        button: {
                            content: "Get Started"
                            styles: {
                                background: $primary
                                color: "#FFFFFF"
                                border_radius: 8dp
                                padding: 12dp 24dp
                                margin_top: 16dp
                            }
                            on_click: log.info("Button tapped!")
                        }
                    ]
                }
            ]
        }
    ]
}
```

## Building

Build your app for both platforms:

```bash
frame build
```

This invokes the Frame compiler pipeline:
1. Parse all `.fr` files with the pest grammar
2. Resolve imports, components, and path aliases
3. Type-check the entire project
4. Generate platform-specific source code (Kotlin + Compose for Android, Swift + UIKit for iOS)
5. Invoke the platform build tools (Gradle for Android, Xcode build for iOS)

Build for a specific platform:

```bash
frame build android
frame build ios
```

## Running Tests

Frame has a built-in test framework. Run tests with:

```bash
frame test
```

Tests are defined in `*.test.fr` files using the `describe`/`it` DSL:

```fr
describe: "App" => {
    it: "starts with correct title" => {
        expect: "myApp" .toBe: "myApp"
    }
}
```

## Deploying

Deploy to a connected device or simulator:

```bash
# iOS (macOS with Xcode required)
frame deploy ios

# Android (Android SDK required)
frame deploy android
```

## Hot-Reload Preview

Start the hot-reload server for instant preview during development:

```bash
frame preview
```

This starts a WebSocket server that watches your source directory. When you save changes to a `.fr` file, Frame recompiles the affected module and pushes an incremental update to any connected simulators or devices. You see your changes without restarting the app.

## IDE Support

### VS Code

Install the **Frame** extension from the VS Code marketplace. The extension activates the built-in LSP server and provides:

- Syntax highlighting
- Code completion
- Diagnostics (errors and warnings inline)
- Hover information
- Go-to-definition
- Code actions

Open your project in VS Code:

```bash
code myApp
```

The LSP server starts automatically when you open a `.fr` file.

### Other Editors

Any LSP-compatible editor (Neovim, Emacs, Helix, etc.) can connect to the Frame LSP server. Start the LSP server manually:

```bash
frame lsp
```

Configure your editor to launch `frame lsp` as the language server for `.fr` files.

## Next Steps

- Learn the Frame language in the [Language Guide](language-guide/overview.md).
- Browse the [Component Reference](component-reference/overview.md) for all 71+ built-in components.
- Explore the [example apps](https://github.com/frame-lang/frame/tree/main/examples) in the repository.
