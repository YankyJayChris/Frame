# Frame Syntax

Syntax highlighting, code snippets, and language support for Frame `.fr` files.

## Features

- **Syntax highlighting** for all built-in components (text, button, column, row, scaffold, app_bar, sidebar, floating_action_button, and 50+ more)
- **Syntax highlighting** for Frame keywords (page, component, fn, :store, :obj, :enum, :vars, :breakpoints, etc.)
- **Syntax highlighting** for style properties, events, string interpolation, and types
- **Code snippets** for all built-in components, declarations, control flow, and more
- **Language configuration** with auto-closing pairs, bracket matching, and indentation rules

## Snippets

### Components
| Prefix | Description |
|--------|-------------|
| `page` | Create a page with route and children |
| `scaffold` | Add a scaffold layout |
| `app_bar` | Add an app bar with title and actions |
| `sidebar` | Add a sidebar navigation panel |
| `fab` / `floating_action_button` | Add a floating action button |
| `column` | Add a column layout |
| `row` | Add a row layout |
| `text` | Add a text element |
| `button` | Add a button |
| `image` | Add an image |
| `icon` | Add an icon |
| `card` | Add a card component |
| `input` | Add a text input |
| `form` | Add a form |
| `list` | Add a list with dynamic items |
| `scroll_view` | Add a scrollable container |
| `divider` | Add a horizontal divider |
| `spacer` | Add a spacer |
| `stack` | Add a stack (layered positioning) |
| `grid` | Add a grid layout |
| `modal` | Add a modal dialog |
| `progress_bar` | Add a linear progress bar |
| `progress_circle` | Add a circular progress indicator |
| `switch` | Add a toggle switch |
| `slider` | Add a slider control |
| `stepper` | Add a stepper |
| `badge` | Add a notification badge |
| `chip` | Add a compact chip |
| `tag` | Add a static tag pill |
| `avatar` | Add a circular avatar |
| `skeleton` | Add a loading skeleton |
| `search_bar` | Add a search bar |
| `tab_bar` | Add a tab bar |
| `rating` | Add a star rating |
| `toast` | Add a toast notification |
| `web_view` | Add an embedded web view |
| `map_view` | Add a map view |

### Declarations
| Prefix | Description |
|--------|-------------|
| `component` | Create a reusable component |
| `:store` | Create a state management store |
| `:obj` | Define an object type |
| `:enum` | Define an enum type |
| `fn` | Define a function |
| `:var` | Declare a local variable |
| `:vars` | Define design tokens |
| `:breakpoints` | Define responsive breakpoints |
| `:i18n` | Add internationalization strings |
| `:typography` | Define typography styles |
| `:validation` | Define validation rules |

### Statements
| Prefix | Description |
|--------|-------------|
| `if` | Add an if/else statement |
| `for` | Add a for loop |
| `try` | Add a try/catch block |
| `switch` | Add a switch statement |
| `fetch` | Add an HTTP fetch call |
| `navigate` | Navigate to another page |
| `import` | Import from a module |

## Installation

1. Open VS Code
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Windows/Linux)
3. Run `Extensions: Install from VSIX...`
4. Select the `.vsix` file

Or copy the `frame-syntax` folder to `~/.vscode/extensions/`.

## Development

To build the extension:

```bash
npm install -g vsce
vsce package
```

This generates a `.vsix` file you can install in VS Code.
