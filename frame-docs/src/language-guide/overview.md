# Frame Language Overview

Frame is a declarative, strictly-typed language for building cross-platform mobile apps. It compiles to native Kotlin (Android) and Swift (iOS).

## File Extension

Frame source files use the `.fr` extension.

## Syntax Style

Frame uses a brace-delimited, indentation-friendly syntax. Indentation is conventional but not semantically significant.

```fr
// Components use colon syntax with braces
column: {
    styles: { padding: 16dp  gap: 8dp }
    children: [
        text: { content: "Hello, World!" }
        button: { content: "Click Me"  on_click: handlePress() }
    ]
}
```

## Comments

```fr
// Single-line comment

/* Multi-line
   block comment */
```

## String Interpolation

Two equivalent syntaxes are supported inside strings:

```fr
let name = "Alice"
let greeting = "Hello, \(name)!"     // paren interpolation
let url = "https://example.com/users/${id}/posts"  // brace interpolation
```

## Types

The type system includes primitive and compound types:

| Type     | Description                          |
|----------|--------------------------------------|
| `string` | UTF-8 text                           |
| `int`    | Signed 64-bit integer                |
| `float`  | 64-bit floating point                |
| `bool`   | Boolean (`true` / `false`)           |
| `object` | Key-value map                        |
| `list`   | Ordered collection                   |
| `void`   | No return value                      |
| `null`   | Nullable value marker                |

Optional types are denoted with `?` suffix: `string?`, `int?`

User-defined types via `:type`, `:enum`, and `:obj`.

## Top-Level Constructs

A `.fr` file can contain any of the following top-level declarations:

| Construct            | Keyword       | Description                              |
|----------------------|---------------|------------------------------------------|
| Page                 | `page:`       | A routed screen                          |
| Component            | `component`   | A reusable UI component                  |
| Function             | `fn`          | A named function (sync or async)         |
| Store                | `:store`      | State management slice                   |
| Object               | `:obj`        | A typed data model                       |
| Enum                 | `:enum`       | Enumeration with optional values         |
| Type alias           | `:type`       | Type alias declaration                   |
| Import               | `import`      | Import from other files or plugins       |
| Constant             | `const`       | Compile-time constant                    |
| Variables            | `:vars`       | Design token variables                   |
| Breakpoints          | `:breakpoints`| Responsive breakpoint definitions        |
| Typography           | `:typography` | Typography scale definitions             |
| I18n                 | `:i18n`       | Internationalization strings             |
| Validation           | `:validation` | Validation schemas                       |
| App lifecycle        | `:app`        | App-level lifecycle hooks                |
| Test suite           | `describe:`   | Test suites                              |

## Naming Conventions

| Category         | Convention   | Example           |
|------------------|--------------|-------------------|
| Built-in components | lowercase  | `text`, `button`, `column` |
| User components  | PascalCase   | `UserCard`, `Header` |
| Functions        | camelCase    | `handlePress`, `loadData` |
| Variables        | camelCase    | `userName`, `isLoading` |

## Keywords

```
page, component, fn, if, else, for, in, switch, case, default,
return, try, catch, import, show_if, build, wait, async, animate,
positioned, describe, it, expect, mock, navigate, navigate_back,
navigate_modal, navigate_dismiss, navigate_back_to, plugin, const
```

## Built-in Components (71+)

All built-in component names are lowercase keywords. Major categories:

- **Layout**: `row`, `column`, `container`, `stack`, `scaffold`, `card`, `divider`, `spacer`, `scroll_view`, `list`, `grid`, `form`, `accordion`, `timeline`, `item`
- **Navigation**: `app_bar`, `bottom_navigation_bar`, `sidebar`, `floating_action_button`, `tab_bar`, `tab`, `bottom_sheet`, `modal`
- **Text & Content**: `text`, `button`, `icon`, `image`, `avatar`, `badge`, `chip`, `tag`, `banner`, `skeleton`, `table`
- **Input**: `input`, `text_area`, `dropdown`, `switch`, `checkbox`, `radio`, `slider`, `stepper`, `search_bar`, `date_picker`, `time_picker`, `color_picker`, `rating`, `otp_input`
- **Feedback**: `toast`, `tooltip`, `progress_bar`, `progress_circle`
- **Media**: `video_player`, `audio_player`, `lottie`, `web_view`, `map_view`, `camera_view`, `qr_scanner`
- **Gesture**: `swipeable`, `draggable`, `refresh`, `long_press`
- **Plugin**: `plugin`
