# Project Structure

A Frame project follows a standard directory layout:

```
myApp/
├── frame.config.json            # Project configuration
├── frame-icons.json             # Registered icon manifest
├── src/
│   ├── project.fr               # Entry point (:vars, :app, imports, top-level fns)
│   ├── pages/                   # Page definitions (routed screens)
│   │   ├── home.fr
│   │   └── profile.fr
│   ├── components/              # Reusable components
│   │   ├── Header.fr
│   │   └── UserCard.fr
│   ├── stores/                  # State management slices
│   │   └── user_store.fr
│   ├── objects/                 # Data model definitions
│   │   └── models.fr
│   ├── enums/                   # Enum definitions
│   │   └── status.fr
│   ├── controllers/             # Business-logic functions
│   │   └── user_controller.fr
│   └── tests/                   # Test suites (*.test.fr)
│       └── user_card.test.fr
├── assets/
│   ├── icons/                   # SVG icons & .frameicons bundle files
│   │   └── default.frameicons
│   ├── images/                  # Image assets
│   │   └── logo.svg
│   └── fonts/                   # TTF/OTF font files
│       └── Inter-Regular.ttf
├── frame_modules/               # Installed plugins
└── build/                       # Generated native projects
    ├── android/                 # Kotlin/Compose project
    └── ios/                     # UIKit/Swift project
```

## Directory Breakdown

| Path | Purpose |
|------|---------|
| `src/project.fr` | Entry point — contains `:vars`, `:app`, `:breakpoints`, `:typography`, top-level `import`s, and global `fn` definitions |
| `src/pages/` | Page declarations — each file defines one or more `page:` blocks with route, params, lifecycle hooks, and children |
| `src/components/` | Reusable `component` definitions with typed `props`, `state`, `styles`, and `children` |
| `src/stores/` | `:store` state management slices with typed fields, default values, actions, and optional persist strategies |
| `src/objects/` | `:obj` data model declarations for domain entities and API response shapes |
| `src/enums/` | `:enum` declarations with optional associated values |
| `src/controllers/` | Business-logic functions that operate on stores and coordinate data flow |
| `src/tests/` | Test suite files using `describe:`, `it:`, `expect:`, and `mock:` |
| `assets/icons/` | Icon assets — `.frameicons` JSON bundle files and individual SVG icons |
| `assets/images/` | Image assets referenced by `image:` components |
| `assets/fonts/` | Custom font files used via `font_family` style property |
| `frame_modules/` | Installed plugins from the registry |
| `build/` | Generated native projects — one per platform |

## Entry Point: `src/project.fr`

Every Frame project must have a `src/project.fr` file. This is where the parser starts.

```fr
:vars {
    $primary:   "#3584e4"
    $surface:   "#ffffff"
    $spacing:   16dp
}

:breakpoints { sm: 360dp  md: 600dp  lg: 900dp  xl: 1200dp }

:app {
    on_launch:     appInit
    on_foreground: appForeground
    on_background: appBackground
}

import { text, button, column, scaffold, app_bar } "frame-core"
import { UserCard }  "@/components/UserCard"
import { UserStore }  "@/stores/user_store"

page: {
    name: "Home"
    route: "/"
    styles: { safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: { title: "MyApp" }
                column: {
                    styles: { padding: 16dp }
                    children: [
                        text: { content: "Welcome!" }
                    ]
                }
            ]
        }
    ]
}
```

## `frame.config.json`

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
    "frame_camera": "0.1.0"
  }
}
```

| Field | Description |
|-------|-------------|
| `name` | Project display name |
| `bundle_id` | Reverse-domain app identifier |
| `version` | Semantic version |
| `build_number` | Incremental build number |
| `render_mode` | `native` (default) or `experimental_skia` |
| `min_android_sdk` | Minimum Android API level |
| `min_ios` | Minimum iOS version |
| `paths` | Path alias mappings (e.g. `"@": "./src"`) |
| `plugins` | Plugin dependencies with versions |

## `@/` Path Alias

The `paths` field in `frame.config.json` enables the `@/` import alias. With `"@": "./src"`, the path `@/components/Header` resolves to `<project_root>/src/components/Header.fr`.

```fr
// Relative import
import { UserCard } "../components/UserCard"

// @/ alias import (resolved relative to project root)
import { Header }   "@/components/Header"
import { UserStore } "@/stores/user_store"
import { Status }    "@/enums/status"
```

The alias root is resolved relative to the directory containing `frame.config.json`.
