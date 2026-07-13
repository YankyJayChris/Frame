# Icon System

Frame's icon system supports three tiers of icon definition, giving you flexibility from quick prototyping to fully custom branded icons.

## Tier 1: Platform Icons (Built-in)

Use any Apple SF Symbol (iOS) or Material Design Icon (Android) directly by name:

```fr
icon: { name: "heart"  styles: { color: "#FF0000"  width: 24  height: 24 } }
icon: { name: "magnifyingglass" }
icon: { name: "gearshape" }
icon: { name: "house.fill" }
icon: { name: "person.fill" }
icon: { name: "plus" }
icon: { name: "trash.fill" }
icon: { name: "bell.fill" }
icon: { name: "star.fill" }
icon: { name: "xmark" }
icon: { name: "chevron.left" }
icon: { name: "checkmark" }
```

## Tier 2: Icon Bundle Files (.frameicons)

Create `.frameicons` JSON files in `assets/icons/` to define custom icon mappings. Each bundle maps logical icon names to platform-specific identifiers and optional SVG path data.

```json
{
  "version": "1.0",
  "icons": [
    {
      "name": "home",
      "sf_symbol": "house.fill",
      "material": "Home",
      "svg_path": "M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z",
      "category": "navigation",
      "tags": ["ui", "nav"]
    }
  ]
}
```

### Bundle Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | **Yes** | Logical icon name (used as `name:` prop value) |
| `sf_symbol` | String | No | Apple SF Symbol name for iOS |
| `material` | String | No | Material Icon name for Android |
| `svg_path` | String | No | SVG path data for custom rendering |
| `category` | String | No | Icon category for organization |
| `tags` | [String] | No | Tags for filtering |

## Tier 3: Custom SVG Icons

Add individual SVG icons that get registered in `frame-icons.json`:

```bash
frame icon add path/to/custom-icon.svg
frame icon add path/to/icon.svg --name "my_custom_icon"
```

This extracts the SVG `<path d="...">` data and registers it in your project's `frame-icons.json` manifest.

## Icon Asset Generation

At deploy time, icons with `svg_path` data are automatically converted to platform-native assets:

- **iOS**: SVG path data wrapped in a reference file at `Assets.xcassets/Resources/`
- **Android**: XML VectorDrawable files generated in `res/drawable/ic_{name}.xml`

```bash
# Manual generation (runs automatically during deploy)
frame icon generate
frame icon generate --target ios
frame icon generate --target android
```

## 330+ Built-in Icons

Frame ships with **330+ pre-defined icons** organized into 14 categories, ready to use by name:

| Category | Count | Examples |
|----------|-------|---------|
| Actions | 61 | add, edit, delete, save, share, download, upload, refresh |
| UI | 95 | menu, grid, list, table, clock, calendar, dashboard |
| Navigation | 14 | home, back, forward, arrow_up, arrow_down, menu |
| Media | 24 | play, pause, stop, camera, photo, video, music |
| Communication | 16 | mail, chat, phone, send, comment, announcement |
| Social | 11 | user, users, group, public, emoji, community |
| Devices | 34 | phone, tablet, laptop, watch, tv, printer, bluetooth |
| Status | 17 | check, error, warning, info, help, verified, priority |
| Commerce | 17 | cart, credit_card, wallet, tag, shopping_bag |
| Files | 4 | folder, file, image, cloud |
| Security | 12 | lock, key, shield, fingerprint, faceid, password |
| Weather | 10 | sun, moon, rain, snow, wind, thunderstorm |
| Health | 10 | heart_rate, pulse, sleep, fitness, nutrition |
| Food | 7 | coffee, tea, restaurant, cake, pizza |

## Icon CLI Commands

| Command | Description |
|---------|-------------|
| `frame icon add <path>` | Register an SVG icon |
| `frame icon add <path> --name <name>` | Register with custom name |
| `frame icon load-bundle <path>` | Load icons from .frameicons bundle |
| `frame icon list` | List all registered icons |
| `frame icon generate` | Generate platform icon assets (PDF for iOS, XML for Android) |
| `frame icon generate --target ios` | Generate only iOS assets |
| `frame icon generate --target android` | Generate only Android assets |

## Icon Lookup Table

Frame maintains a `frame-icon-lookup.json` file in the project root that maps all available icon names to their platform identifiers. This is used by the LSP server for icon name completions and by the VS Code extension's icon browser.

## Bundle File Loading

Multiple bundle files can be placed in `assets/icons/` — the system loads all of them and merges by name (first definition wins):

```
myApp/assets/icons/
├── default.frameicons       # 332 bundled icons (shipped with Frame)
├── custom.frameicons        # Your custom icon definitions
└── imported_icons.frameicons # Icons from `frame icon load-bundle`
```
