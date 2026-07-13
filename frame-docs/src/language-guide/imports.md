# Imports

Frame uses the `import` keyword to bring in symbols from other files, built-in components, and plugins.

## Importing Built-in Components

Built-in components from `frame-core` do not require a path — they are available globally:

```fr
import { text, button, column, scaffold, app_bar } "frame-core"
import { row, stack, container, card, divider } "frame-core"
import { icon, image, avatar, badge, chip, tag } "frame-core"
import { input, text_area, dropdown, switch, checkbox } "frame-core"
import { list, grid, form, scroll_view, spacer } "frame-core"
```

## Importing from Relative Paths

```fr
import { UserCard }  "../components/UserCard"
import { Header }    "./components/Header"
import { loadUser }  "../../controllers/UserController"
import { UserStore } "../stores/user_store"
```

File extensions (`.fr`) are optional:

```fr
import { UserCard }  "../components/UserCard.fr"
import { UserStore } "../stores/user_store.fr"
```

## Importing with `@/` Path Alias

The `@` alias is configured in `frame.config.json` under `paths`:

```json
{
  "paths": {
    "@": "./src"
  }
}
```

```fr
import { UserCard }  "@/components/UserCard"
import { Header }    "@/components/Header"
import { UserStore } "@/stores/user_store"
import { Status }    "@/enums/status"
import { User }      "@/objects/models"
```

## Importing with Aliases

Use the `as` keyword to rename imports:

```fr
import { UserCard as Card } "../components/UserCard"
import { UserStore as Store } "../stores/user_store"
import { Button as PrimaryButton, Text } "frame-core"
```

## Importing from Plugins

Plugin imports use the `frame-` prefix:

```fr
import { capture }         "frame-camera"
import { isOnline }        "frame-connectivity"
import { saveFile }        "frame-storage"
import { showMap }         "frame-maps"
import { scanQR }          "frame-qr"
```

Plugins are installed in `frame_modules/` via the CLI:

```bash
frame plugin add frame_camera
frame plugin add frame_storage
frame plugin add frame_connectivity
```

## Re-exports

Use `export` to re-export symbols from other files:

```fr
export { UserCard }    "../components/UserCard"
export { UserStore }   "../stores/user_store"
export { Status }      "../enums/status"
```

## Multiple Imports from a Single File

```fr
import { UserCard, ProfileCard, SettingsCard } "@/components/cards"
import { UserStore, AppStore, ThemeStore } "@/stores"
import { Status, Color, Role } "@/enums"
```

## Import Resolution

| Pattern | Resolution |
|---------|------------|
| `"frame-core"` | Built-in runtime components — no file resolution |
| `"frame-*"` | Plugin — resolved from `frame_modules/frame_<name>/src/index.fr` |
| `"./path"` | Relative to the current file |
| `"../path"` | Relative parent path |
| `"@/path"` | Resolved via `paths.@` in `frame.config.json` |

## Import Rules

- Imports are hoisted to the top of the file (can appear anywhere).
- Circular dependencies are detected and reported as errors.
- Unused imports produce a compiler warning.
- Duplicate imports are merged automatically.
