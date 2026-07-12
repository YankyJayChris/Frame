# Frame

**Frame** is a cross-platform mobile framework with a rich, strictly-typed declarative language (`.fr`). Write UI once — compile to native Kotlin (Android) and Swift (iOS).

- **Declarative**: Describe what your UI looks like, not how to build it
- **Strictly-typed**: Catch errors at compile time, not runtime
- **57+ built-in components**: Layout, sidebar, FAB, input, navigation, media, feedback, gestures — everything you need
- **Native**: Compiles to real Kotlin/Compose and UIKit/Swift — no runtime, no WebView
- **Plugin system**: Extend with community or private plugins (`$@user/repo`)
- **Hot-reload**: Instant preview during development

---

## Table of Contents

- [Quick Start](#quick-start)
- [Language Guide](#language-guide)
  - [Project Structure](#project-structure)
  - [Pages](#pages)
  - [Components](#components)
  - [Styles](#styles)
  - [Variables](#variables)
  - [Functions](#functions)
  - [String Interpolation](#string-interpolation)
  - [Named Arguments](#named-arguments)
  - [Fetch & Headers](#fetch--headers)
  - [Validation](#validation)
  - [Enums](#enums)
  - [Type Aliases](#type-aliases)
  - [Imports](#imports)
  - [Store / State Management](#store--state-management)
  - [Built-in Functions](#built-in-functions)
  - [Logging](#logging)
  - [Events & Lifecycle](#events--lifecycle)
  - [Navigation](#navigation)
  - [App Lifecycle](#app-lifecycle)
  - [Conditionals & Lists](#conditionals--lists)
- [Component Reference](#component-reference)
- [Plugin System](#plugin-system)
- [CLI Reference](#cli-reference)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

---

## Quick Start

### Installation

**From source (recommended during development):**

```bash
# Clone and build
git clone https://github.com/frame-lang/frame.git
cd frame
cargo build --release

# Add to PATH so `frame` works as a global command
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/frame" ~/.local/bin/frame

# Make it permanent — add to your shell profile (~/.zshrc, ~/.bashrc, etc.)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Verify
export PATH="$HOME/.local/bin:$PATH" && frame --version
# → frame 0.1.0
```

**Via Cargo (installs globally, takes a few minutes):**

```bash
cargo install --path .

# Verify
frame --version
# → frame 0.1.0
```

 **Note:** After installation, `frame` is available from any directory. If your shell doesn't find it, make sure `~/.local/bin` (or `~/.cargo/bin` for cargo install) is in your `PATH`:
```bash
 export PATH="$HOME/.local/bin:$PATH"   # for ln -sf install
 export PATH="$HOME/.cargo/bin:$PATH"   # for cargo install
```

---

### Create and run your first app

```bash
# Scaffold a new project (MVC or Clean Architecture)
frame start myApp
cd myApp

# Verify the build environment
frame check

# Compile .fr source files
frame build

# Run tests
frame test

# Deploy to Android (requires Android SDK)
frame deploy android

# Deploy to iOS (requires Xcode, macOS only)
frame deploy ios

# Hot-reload dev server
frame preview
```

---

### Try the example apps

The repository ships two ready-to-run example apps. After installing `frame`:

```bash
# Regenerate both example projects with all latest features
frame init-examples

# Run the MVC blog-app example
cd examples/blog-app
frame build          # compiles — outputs to build/android/ and build/ios/
frame test           # runs UserStore, API, and navigation test suites
frame deploy android
frame deploy ios

# Run the Clean Architecture profile example
cd ../profile
frame build
frame test
frame deploy android
frame deploy ios
```

```
myApp/
├── pages/
│   └── Home.fr          # Page definitions (routes, UI)
├── components/
│   └── UserCard.fr       # Reusable components
├── stores/
│   └── UserModel.fr      # State management
├── frame.config.json     # Project configuration
└── frame.lock            # Plugin lock file
```

### Pages

Pages are the entry points of your app. Each page has a route, optional lifecycle hooks, and a UI tree.

```frame
page: {
    name: "Home"
    route: "/"

    styles: {
        safe_area: true
        background: "#FFFFFF"
    }

    children: [
        scaffold: {
            styles: { safe_area: true }
            children: [
                app_bar: { title: "My App" }
                column: {
                    styles: { padding: 16 }
                    children: [
                        text: { content: "Welcome!" }
                    ]
                }
            ]
        }
    ]
}
```

### Components

Components are reusable building blocks. They can have props (with defaults), styles, state, events, animations, and children.

```frame
component UserCard => (
    name: String,
    email: String = "no-email@example.com",
    age?: Int
) {
    styles: {
        background: "#FFFFFF"
        border_radius: 8
        padding: 16
        margin_bottom: 8
    }

    props: {
        count: Int = 0
    }

    state: {
        expanded: Bool = false
    }

    children: [
        row: {
            children: [
                avatar: { src: "https://..." }
                column: {
                    styles: { margin_left: 12 }
                    children: [
                        text: { content: name }
                        text: { content: email }
                    ]
                }
            ]
        }
    ]

    events: {
        on_click: toggleExpanded()
    }

    animate: {
        expanded: { duration: 300 }
    }
}
```

**Prop rules:**
- `name: Type` — required prop
- `name: Type = default` — optional with default
- `name?: Type` — truly optional (nullable)

### Styles

Every component supports a `styles:` block. Over 30 style properties are available:

| Category       | Properties                                                                                                                                                     |
| ----------------| ----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Layout**     | `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height`, `x`, `y`, `flex`, `flex_wrap`, `direction`, `align`, `justify`, `gap`, `aspect_ratio` |
| **Spacing**    | `margin`, `margin_top`, `margin_bottom`, `margin_left`, `margin_right`, `padding`, `padding_top`, `padding_bottom`, `padding_left`, `padding_right`            |
| **Appearance** | `background`, `color`, `font_size`, `font_weight`, `font_family`, `border`, `border_radius`, `opacity`, `visible`                                              |
| **Safe Area**  | `safe_area: true/false` (defaults to `true`)                                                                                                                   |
| **Overflow**   | `overflow: hidden/scroll/visible`, `overflow_x`, `overflow_y`, `clip_behavior`                                                                                 |
| **Text**       | `text_overflow: ellipsis/clip/fade`, `max_lines`, `line_clamp`                                                                                                 |
| **Image**      | `fit: cover/contain/fill/fit_width/fit_height/none`                                                                                                            |
| **Scroll**     | `scroll_indicator`, `scroll_snap: start/center/end/none`, `scroll_enabled`                                                                                     |
| **Events**     | `on_scroll`, `on_scroll_end`                                                                                                                                   |

```frame
container: {
    styles: {
        width: "100%"
        height: 200
        background: "#F5F5F5"
        border_radius: 12
        padding: 16
        overflow: hidden
        safe_area: true
    }
}
```

**Safe area:** The `safe_area: true/false` style property controls whether a component respects system safe areas (status bar, notch, home indicator). This is especially important on `scaffold` and root containers. Default is `true`. On Android, `safe_area: false` renders edge-to-edge. On iOS, the scaffold pins to the parent's `safeAreaLayoutGuide`.

Style values support **responsive breakpoints**:
```frame
styles: {
    width: [100%, @md: 75%, @lg: 50%]
    font_size: [14sp, @tablet: 18sp]
}
```

### Variables

Variables are immutable by default (`:var`) and must be explicitly declared mutable (`:var mut`).

```frame
:var greeting = "Hello"          // inferred String, immutable
:var mut count = 0                // inferred Int, mutable
:var name: String = "World"       // explicit type
:var mut items: List<String> = [] // mutable list
```

**Type inference** works from initializer values. The `:var` keyword makes variables read-only — attempting to reassign produces a compile error. Use `:var mut` for mutable values.

### Functions

```frame
fn greet(name: String) -> String {
    return "Hello, " + name
}

fn fetchData() async {
    let result = wait: fetch("https://api.example.com/data", {
        headers: {
            "Authorization": "Bearer " + token
        }
    })
    :var mut data = result.body
}
```

**Async functions** use `async` keyword and must be called with `wait:`.

### String Interpolation

```frame
:var name = "Alice"
text: { content: "Hello \(name), you have \(count) messages" }
```

Use `\(expr)` for inline expressions and variables.

### Named Arguments

When calling functions or building components, you can use named arguments with defaults:

```frame
fn createUser(name: String, age: Int = 18, email: String = "none") {
    // ...
}

// Call with named args
createUser(name: "Alice", age: 25)
createUser(name: "Bob")  // uses defaults for age and email

// Also works in component props
button: {
    content: "Submit"
    on_click: submitForm(email: userEmail, validate: true)
}
```

### Fetch & Headers

```frame
:var mut response = wait: fetch("https://api.example.com/data", {
    method: "POST"
    headers: {
        "Content-Type": "application/json"
        "Authorization": "Bearer \(token)"
        "X-Custom": customHeader
    }
    body: jsonData
})
```

Headers support both string literals and variable references, with interpolation.

### Validation

Define validation schemas and apply them to inputs:

```frame
:validation SignupForm {
    email: required, email
    password: required, min_length(8)
    age: required, min(18)
}

// Apply inline
input: {
    value: email
    validate: "email"
    on_error: showError("Invalid email")
}

// Apply schema
form: {
    schema: "SignupForm"
    children: [
        input: { value: formName, validate: "required" }
        input: { value: formEmail, validate: "email" }
    ]
}
```

**Built-in validators:** `required`, `email`, `min(n)`, `max(n)`, `min_length(n)`, `max_length(n)`, `pattern(regex)`, `url`, `phone`, `number`, `integer`.

### Enums

```frame
:enum Status {
    Active
    Inactive
    Pending
}

:enum Color with values {
    Red = "#FF0000"
    Green = "#00FF00"
    Blue = "#0000FF"
}
```

### Type Aliases

```frame
:type UserId = Int
:type JsonMap = {String: Any}
:type Callback = (String) -> Bool
```

### Imports

```frame
import { UserCard } from "components/UserCard"
import { formatDate, parseDate } from "utils/dates"
```

### Store / State Management

```frame
:store UserModel {
    fields: {
        token: Token
        username: Local
        preferences: Persistent
    }
    functions: {
        fn login(username: String, password: String) async {
            :var mut result = wait: fetch("/api/login", {
                method: "POST"
                headers: {
                    "Content-Type": "application/json"
                }
                body: to_json({ "user": username, "pass": password })
            })
            token = result.token
        }
    }
}
```

**Field strategies:**
- Default (unmarked): In-memory only
- `Token`: Stored securely (Keychain on iOS, EncryptedSharedPreferences on Android)
- `Local`: Stored locally (UserDefaults on iOS, SharedPreferences on Android)
- `Persistent`: Persisted across sessions

### Built-in Functions

Frame provides a rich standard library through dotted function calls:

#### String Methods
| Function | Android | iOS |
|----------|---------|-----|
| `string.upper(x)` | `x.uppercase()` | `x.uppercased()` |
| `string.lower(x)` | `x.lowercase()` | `x.lowercased()` |
| `string.trim(x)` | `x.trim()` | `x.trimmingCharacters(in: .whitespaces)` |
| `string.contains(x, sub)` | `x.contains(sub)` | `x.contains(sub)` |
| `string.starts_with(x, prefix)` | `x.startsWith(prefix)` | `x.hasPrefix(prefix)` |
| `string.ends_with(x, suffix)` | `x.endsWith(suffix)` | `x.hasSuffix(suffix)` |
| `string.replace(x, a, b)` | `x.replaceFirst(a, b)` | `x.replacingOccurrences(of: a, with: b)` |
| `string.replace_all(x, a, b)` | `x.replace(a, b)` | `x.replacingOccurrences(of: a, with: b)` |
| `string.split(x, sep)` | `x.split(sep)` | `x.components(separatedBy: sep)` |
| `string.join(xs, sep)` | `xs.joinToString(sep)` | `xs.joined(separator: sep)` |
| `string.slice(x, start, end)` | `x.substring(start, end)` | `String(x.prefix(end)).dropFirst(start)` |
| `string.length(x)` | `x.length` | `x.count` |
| `string.is_empty(x)` | `x.isEmpty()` | `x.isEmpty` |
| `string.to_int(x)` | `x.toInt()!!` | `Int(x)!` |
| `string.to_float(x)` | `x.toDouble()!!` | `Double(x)!` |
| `string.pad_left(x, n, c)` | `x.padStart(n, c)` | `padLeft(n, c)` |
| `string.pad_right(x, n, c)` | `x.padEnd(n, c)` | `padRight(n, c)` |

#### Number Functions
| Function | Android | iOS |
|----------|---------|-----|
| `number.abs(x)` | `Math.abs(x)` | `abs(x)` |
| `number.sqrt(x)` | `Math.sqrt(x.toDouble())` | `sqrt(x)` |
| `number.floor(x)` | `kotlin.math.floor(x)` | `floor(x)` |
| `number.ceil(x)` | `kotlin.math.ceil(x)` | `ceil(x)` |
| `number.round(x)` | `kotlin.math.round(x)` | `round(x)` |
| `number.min(a, b)` | `Math.min(a, b)` | `min(a, b)` |
| `number.max(a, b)` | `Math.max(a, b)` | `max(a, b)` |
| `number.clamp(v, lo, hi)` | `v.coerceIn(lo, hi)` | `min(max(v, lo), hi)` |
| `number.random()` | `Math.random()` | `Double.random(in: 0..<1)` |

#### List/Collection Methods
| Function | Android | iOS |
|----------|---------|-----|
| `list.length(xs)` | `xs.size` | `xs.count` |
| `list.push(xs, item)` | `xs.add(item)` | `xs.append(item)` |
| `list.contains(xs, item)` | `xs.contains(item)` | `xs.contains(item)` |
| `list.is_empty(xs)` | `xs.isEmpty()` | `xs.isEmpty` |
| `list.at(xs, i)` | `xs[i]` | `xs[i]` |
| `list.first(xs)` | `xs.first()` | `xs.first!` |
| `list.last(xs)` | `xs.last()` | `xs.last!` |
| `list.reverse(xs)` | `xs.reversed()` | `xs.reversed()` |
| `list.sum(xs)` | `xs.sum()` | `xs.reduce(0, +)` |
| `list.average(xs)` | `xs.average()` | `Double(xs.reduce(0, +)) / Double(xs.count)` |

#### Math Functions
| Function | Android | iOS |
|----------|---------|-----|
| `math.abs(x)` | `Math.abs(x)` | `abs(x)` |
| `math.sqrt(x)` | `Math.sqrt(x)` | `sqrt(x)` |
| `math.sin(x)` | `Math.sin(x)` | `sin(x)` |
| `math.cos(x)` | `Math.cos(x)` | `cos(x)` |
| `math.tan(x)` | `Math.tan(x)` | `tan(x)` |
| `math.pi` | `Math.PI` | `Double.pi` |
| `math.pow(a, b)` | `Math.pow(a, b)` | `pow(a, b)` |
| `math.log(x)` | `Math.log(x)` | `log(x)` |

#### Date Functions
| Function | Android | iOS |
|----------|---------|-----|
| `date.now()` | `System.currentTimeMillis()` | `Date().timeIntervalSince1970` |
| `date.format(d, fmt)` | `SimpleDateFormat(fmt).format(d)` | `DateFormatter().string(from: d)` |

#### Object/Map Functions
| Function | Android | iOS |
|----------|---------|-----|
| `object.keys(dict)` | `dict.keys.toList()` | `Array(dict.keys)` |
| `object.values(dict)` | `dict.values.toList()` | `Array(dict.values)` |
| `object.has_key(dict, key)` | `dict.containsKey(key)` | `dict.keys.contains(key)` |

#### JSON
| Function | Android | iOS |
|----------|---------|-----|
| `from_json(s)` | `Gson().fromJson(s, Map::class.java)` | `JSONSerialization...` |
| `to_json(obj)` | `Gson().toJson(obj)` | `JSONSerialization...` |

#### Utility Functions
| Function | Android | iOS |
|----------|---------|-----|
| `util.print(x)` | `println(x)` | `print(x)` |
| `util.type_of(x)` | `x::class.simpleName` | `type(of: x)` |
| `util.is_null(x)` | `x == null` | `x == nil` |
| `util.is_not_null(x)` | `x != null` | `x != nil` |
| `util.uuid()` | `UUID.randomUUID().toString()` | `UUID().uuidString` |
| `util.encode_base64(s)` | `Base64...` | `Data...` |
| `util.decode_base64(s)` | `String(Base64...)` | `String(data:...)` |
| `util.encode_url(s)` | `URLEncoder.encode(s, "UTF-8")` | `s.addingPercentEncoding...` |
| `util.decode_url(s)` | `URLDecoder.decode(s, "UTF-8")` | `s.removingPercentEncoding!` |

### Logging

Frame provides structured logging via `log.info()`, `log.warn()`, `log.error()`, `log.debug()`, and `log.verbose()`:

```frame
log.info("App started successfully")
log.warn("Cache miss for user \(userId)")
log.error("Failed to load data: \(errorMessage)")
log.debug("Button pressed at position \(x), \(y)")
```

These compile to platform-native logging:
- **Android**: `android.util.Log.i("Frame", "message")` / `Log.w()` / `Log.e()` / `Log.d()` / `Log.v()`
- **iOS**: `os_log(.info, "{message}")` / `os_log(.default, ...)` / `os_log(.error, ...)`

Log levels follow standard conventions:
| Function | Level | When to use |
|----------|-------|-------------|
| `log.verbose(...)` | VERBOSE | Detailed debugging, high volume |
| `log.debug(...)` | DEBUG | Debugging information |
| `log.info(...)` | INFO | General operational info |
| `log.warn(...)` | WARN | Unexpected but recoverable issues |
| `log.error(...)` | ERROR | Errors that need attention |

### Events & Lifecycle

**Events** are attached to component properties:

```frame
button: {
    content: "Submit"
    on_click: handleSubmit()
}

input: {
    value: email
    on_change: validateEmail()
    on_focus: logFocus()
    on_blur: logBlur()
}
```

**Component lifecycle hooks** (`on_mount`, `on_update`, `on_unmount`, `watch`):

```frame
column: {
    on_mount:   loadInitialData()   // fires once after first render
    on_update:  refreshList()       // fires whenever `watch` dependency changes
    watch:      UserStore.items     // dependency for on_update
    on_unmount: cancelRequests()    // fires when node is removed
    children: [...]
}
```

**Page lifecycle hooks:**

```frame
page: {
    name: "Dashboard"
    route: "/dashboard"
    before_enter: checkAuth()       // guard — called before transition (viewWillAppear / LaunchedEffect)
    on_mount:     loadData()        // called when fully visible (viewDidAppear / LaunchedEffect "mount")
    before_leave: saveState()       // called as page leaves (viewDidDisappear / DisposableEffect)
    on_unmount:   cleanup()         // combined with before_leave on dispose
    on_foreground: resumePolling()  // app came to foreground (ON_RESUME / willEnterForeground)
    on_background: pausePolling()   // app went to background (ON_PAUSE / didEnterBackground)
}
```

**App-level lifecycle** (declared once in `project.fr`):

```frame
:app {
    on_launch:     initAnalytics    // Application.onCreate / didFinishLaunching
    on_foreground: resumeSession    // ProcessLifecycleOwner ON_START / sceneWillEnterForeground
    on_background: persistState     // ProcessLifecycleOwner ON_STOP / sceneDidEnterBackground
}
```

**Navigation:**

```frame
// Push a new screen
navigate("/profile")

// Push with typed params and options
navigate("/profile/\(userId)", replace: true)
navigate("/home", clear_stack: true)
navigate("/detail", transition: "slide_up")
navigate("/detail", single_top: true)

// Replace current entry (no new back-stack entry)
navigate_replace("/login")

// Go back
navigate_back()

// Go back to a specific route
navigate_back_to("/home")

// Present modally
navigate_modal("/settings")
navigate_dismiss()
```

**Page with typed route params:**

```frame
page: {
    name: "Profile"
    route: "/profile/:userId"
    params: { userId: string }       // typed — generates typed Screen/ViewController params
    before_enter: checkAuth()
    children: [
        text: { content: userId }    // param available as a state-like variable
    ]
}
```

| Function | Description | Android | iOS |
|----------|-------------|---------|-----|
| `navigate(route)` | Push route onto stack | `navController.navigate(route)` | `pushViewController(routeVC(for:))` |
| `navigate(route, replace: true)` | Replace current entry | `navOptions { popUpTo ... }` | `popViewController` + `push` |
| `navigate(route, clear_stack: true)` | Clear stack, then push | `popUpTo(startDest) { inclusive=true }` | `setViewControllers([vc])` |
| `navigate(route, single_top: true)` | Avoid duplicate screens | `launchSingleTop = true` | Guard on push |
| `navigate(route, transition: "slide_up")` | Custom transition animation | Compose `enterTransition` | Custom `UIViewControllerAnimatedTransitioning` |
| `navigate_replace(route)` | Replace without back entry | `popUpTo(current) { inclusive=true }` | Pop + push without animation |
| `navigate_back()` | Pop one entry | `navController.popBackStack()` | `popViewController(animated: true)` |
| `navigate_back_to(route)` | Pop to named route | `popBackStack(route, inclusive=false)` | `popToViewController` via `FrameRoutable` |
| `navigate_modal(route)` | Present modally | `navController.navigate(route)` | `present(routeVC(for:))` |
| `navigate_dismiss()` | Dismiss modal | `navController.popBackStack()` | `dismiss(animated: true)` |


### Conditionals & Lists

```frame
// Conditional rendering with show_if
text: {
    content: "Hello Admin"
    show_if: isAdmin
}

// Lists with data binding
list: {
    data: items
    build: item {
        text: { content: item.name }
    }
}
```

---

## Component Reference

### Layout Components

#### `row`
Horizontally arranges children.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| *(none)* | | | |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click`, `on_scroll`, `on_scroll_end` | Yes (any) |

```frame
row: {
    styles: { gap: 8, padding: 16 }
    children: [
        text: { content: "Item 1" }
        text: { content: "Item 2" }
    ]
}
```

#### `column`
Vertically arranges children.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click`, `on_scroll`, `on_scroll_end` | Yes (any) |

```frame
column: {
    styles: { gap: 12, padding: 16 }
    children: [
        text: { content: "Title" }
        text: { content: "Body text" }
    ]
}
```

#### `container`
Generic box container.

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

#### `stack`
Positions children relative to each other (z-order).

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `alignment` | String | No | `"topLeft"` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

Values for `alignment`: `topLeft`, `topCenter`, `topRight`, `centerLeft`, `center`, `centerRight`, `bottomLeft`, `bottomCenter`, `bottomRight`.

Children can specify absolute positioning:
```frame
stack: {
    alignment: "center"
    children: [
        image: { src: "bg.jpg" }
        text: { content: "Overlay" }    // positioned: { x: 16, y: 16 }
    ]
}
```

#### `scaffold`
Top-level screen structure with safe area, app bar, and bottom navigation.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles, **`safe_area: true/false`** | — | Yes (any) |

Automatically recognizes `app_bar` and `bottom_navigation_bar` children for proper positioning.

```frame
scaffold: {
    styles: { safe_area: true }
    children: [
        app_bar: { title: "Home" }
        column: {
            children: [ text: { content: "Content" } ]
        }
        bottom_navigation_bar: {
            children: [ ... ]
        }
    ]
}
```

**Safe area:** On Android, `Scaffold` composable applies system window insets automatically. On iOS, the scaffold pins to `safeAreaLayoutGuide`. Set `safe_area: false` for edge-to-edge rendering.

#### `card`
Elevated container with shadow.

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

#### `divider`
Horizontal or vertical separator line.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| `color`, `margin` | — | No |

#### `spacer`
Blank flexible space.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| `width`, `height` | — | No |

#### `scroll_view`
Scrollable container.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_scroll`, `on_scroll_end` | Yes (any) |

#### `list`
Data-driven repeating list.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `data` | Expression | No | — |
| `build` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_scroll`, `on_scroll_end` | Yes (template) |

```frame
list: {
    data: users
    build: u {
        text: { content: u.name }
    }
}
```

#### `grid`
Grid layout with columns.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `columns` | Int | No | — |
| `data` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `form`
Form container with validation schema.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `schema` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_submit` | Yes (any) |

#### `accordion`
Expandable/collapsible section.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `title` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `timeline`
Vertical timeline display.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `item`
Generic list item wrapper.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `sidebar`
See [Navigation Components — sidebar](#sidebar-1).

#### `plugin`
Custom plugin component.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `name` | String | **Yes** | — |
| `method` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| — | — | No |

```frame
plugin: {
    name: "analytics"
    method: "trackEvent"
}
```

---

### Text & Content Components

#### `text`
Displays formatted text.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `content` | String | No | `""` |

| Styles | Events | Children |
|--------|--------|----------|
| `color`, `font_size`, `font_weight`, `font_family`, `text_overflow`, `max_lines`, `line_clamp`, `text_align` | `on_click` | No |

```frame
text: {
    content: "Hello, World!"
    styles: {
        font_size: 18
        font_weight: "bold"
        color: "#333333"
        text_overflow: ellipsis
        max_lines: 2
    }
}
```

#### `button`
Tappable button.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `content` | String | No | `""` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | No |

```frame
button: {
    content: "Submit"
    styles: {
        background: "#007AFF"
        color: "#FFFFFF"
        border_radius: 8
        padding: 12
    }
    on_click: handleSubmit()
}
```

#### `icon`
Displays an SVG icon from the assets.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `name` | String | No | — |
| `path` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| `color`, `font_weight`, `width`, `height`, `opacity`, `margin` | `on_click` | No |

```frame
icon: { name: "heart" styles: { color: "#FF0000" width: 24 height: 24 } }
```

Add custom icons: `frame icon add path/to/icon.svg`

#### `image`
Displays an image from a URL or asset.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `src` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| `width`, `height`, `border_radius`, `opacity`, `margin`, `fit`, `clip_behavior` | `on_click` | No |

`fit` values: `cover`, `contain`, `fill`, `fit_width`, `fit_height`, `none`

```frame
image: {
    src: "https://example.com/photo.jpg"
    styles: {
        width: "100%"
        height: 200
        fit: cover
        border_radius: 12
    }
}
```

#### `avatar`
Circular avatar image.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `src` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| Image styles | `on_click` | No |

#### `badge`
Badge/number indicator.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `count` | Int | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `chip`
Compact chip/tag element.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `content` | String | No | — |
| `label` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | No |

#### `tag`
Visual tag/label.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `content` | String | No | — |
| `label` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `banner`
Prominent banner message.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

#### `skeleton`
Loading placeholder.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `table`
Tabular data display.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `data` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

---

### Action Components

#### `floating_action_button`
See [Navigation Components — floating_action_button](#floating_action_button).

### Input Components

#### `input`
Text input field.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |
| `placeholder` | String | No | — |
| `validate` | Expression | No | — |
| `on_error` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_submit`, `on_focus`, `on_blur` | No |

#### `text_area`
Multi-line text input.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |
| `placeholder` | String | No | — |
| `lines` | Int | No | — |
| `validate` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_submit` | No |

#### `dropdown`
Dropdown/select menu.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |
| `validate` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_select` | Yes (any) |

#### `switch`
Toggle switch.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Bool | No | — |
| `checked` | Bool | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `checkbox`
Checkbox.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Bool | No | — |
| `checked` | Bool | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `radio`
Radio button.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `selected` | Bool | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | No |

#### `slider`
Range slider.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Float | No | — |
| `min` | Float | No | — |
| `max` | Float | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `stepper`
Increment/decrement stepper.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Int | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_increment`, `on_decrement` | No |

#### `search_bar`
Search input.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |
| `placeholder` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_submit` | No |

#### `date_picker`
Date picker.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |
| `validate` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `time_picker`
Time picker.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |
| `validate` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `color_picker`
Color picker.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `rating`
Star rating.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Int | No | — |
| `max` | Int | No | `5` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `otp_input`
One-time password input.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `length` | Int | No | `6` |
| `validate` | Expression | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

---

### Navigation Components

#### `app_bar`
Top app bar/toolbar with leading icon and action items.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `title` | String | No | — |
| `leading` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (icon/button children become action items) |

```frame
app_bar: {
    title: "Home"
    leading: "menu"
    children: [
        icon: { name: "search", on_click: openSearch() }
        icon: { name: "settings", on_click: openSettings() }
    ]
}
```

- `leading`: Icon name for the navigation icon (e.g. hamburger menu, back arrow). On Android renders as `navigationIcon` slot in `TopAppBar`. On iOS renders as `leftBarButtonItem`.
- **Children**: Each child is rendered as a trailing action item. On Android, children render inside the `actions` slot of `TopAppBar`. On iOS, children become `rightBarButtonItems`.
- Designed to be used as a direct child of `scaffold`, which automatically passes it as the `topBar` slot.

#### `sidebar`
Side drawer / side navigation panel.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `side` | String | No | `"left"` |
| `width` | String | No | `"260"` |
| `collapsed` | Bool | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

```frame
row: {
    children: [
        sidebar: {
            side: "left"
            width: "280"
            styles: { background: "#F5F5F5" }
            children: [
                text: { content: "Menu Item 1" }
                text: { content: "Menu Item 2" }
                text: { content: "Menu Item 3" }
            ]
        }
        column: {
            children: [ text: { content: "Main Content" } ]
        }
    ]
}
```

On Android, the sidebar renders as a fixed-width `Column` composable. On iOS, it renders as a `UIStackView` pinned to the leading edge of the parent.

#### `floating_action_button`
Circular action button (FAB). Supports both simple props and child components.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `content` | String | No | — |
| `icon` | String | No | — |
| `position` | String | No | `"bottom_end"` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (component children render inside the FAB) |

```frame
// Simple usage with icon prop
floating_action_button: {
    icon: "plus"
    on_click: handleAdd()
}

// With content label
floating_action_button: {
    content: "Save"
    icon: "checkmark"
    on_click: saveData()
}

// With child components (icon component inside)
floating_action_button: {
    children: [
        icon: { name: "heart", styles: { color: "#FF0000" width: 24 height: 24 } }
    ]
    on_click: handleLike()
}
```

When children are present, they are rendered inside the FAB instead of using the `icon`/`content` props. This allows you to use the `icon:` component (or any other component) as the FAB content.

On Android, renders as Material3 `FloatingActionButton` composable. On iOS, renders as a `UIView` with a centered `UIStackView` for child content, positioned at the bottom-right of the parent.

#### `bottom_navigation_bar`
Bottom tab bar.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

Designed to be used as a direct child of `scaffold`.

#### `tab_bar`
Horizontal tab bar.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `selected` | Int | No | — |
| `current` | Int | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (only `tab` children) |

#### `tab`
Individual tab item.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `content` | String | No | — |
| `icon` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click`, `on_select` | No |

#### `bottom_sheet`
Modal bottom sheet.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_unmount` | Yes (any) |

#### `modal`
Alert dialog or modal.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `title` | String | No | — |
| `message` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_unmount` | Yes (any) |

---

### Feedback Components

#### `toast`
Brief transient notification.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `message` | String | No | — |
| `duration` | Int | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `tooltip`
Contextual tooltip on hover/long-press.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `text` | String | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `progress_bar`
Horizontal progress indicator.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Float | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `progress_circle`
Circular progress indicator.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `value` | Float | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

---

### Media Components

#### `video_player`
Video playback.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `src` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

#### `audio_player`
Audio playback.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `src` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

#### `lottie`
Lottie animation.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `src` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

#### `web_view`
Web content embed.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `url` | String | **Yes** | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `map_view`
Embedded map.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `lat` | Float | No | — |
| `lng` | Float | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `camera_view`
Live camera preview.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `qr_scanner`
QR/barcode scanner.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_scan` | No |

---

### Gesture Components

#### `swipeable`
Detect swipe gestures on child content.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_swipe` | Yes (any) |

#### `draggable`
Make children draggable.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_drag` | Yes (any) |

#### `refresh`
Pull-to-refresh container.

| Prop | Type | Required | Default |
|------|------|----------|---------|
| `refreshing` | Bool | No | — |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_refresh` | Yes (any) |

#### `long_press`
Detect long-press gesture.

| Prop | Type | Required | Default |
|------|------|----------|---------|

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_long_press` | Yes (any) |

---

## Plugins

### Install from GitHub

```bash
frame plugin add @user/repo
frame plugin add @user/repo@v1.2.3  # specific version
```

### Create a Plugin

```bash
frame start myPlugin --plugin
```

A plugin typically provides:
- **Custom components** (`.fr` files)
- **Swift/Kotlin source** (native code)
- **Assets** (icons, fonts)
- **Permissions** (Android/iOS manifest entries)

See [plugins documentation](docs/plugins.md) for details.

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `frame start <name>` | Create new project |
| `frame start <name> --plugin` | Create new plugin |
| `frame build <platform>` | Compile .fr files to native code |
| `frame deploy <platform>` | Deploy to device/simulator |
| `frame test` | Run tests |
| `frame preview` | Start hot-reload preview server |
| `frame lint` | Run static analysis |
| `frame check` | Type-check without emitting code |
| `frame icon add <path>` | Add SVG icon to assets |
| `frame plugin add <ref>` | Install plugin from GitHub |
| `frame init-examples` | Generate example files |

---

## Architecture

```
.fr files  ──→  PEG Parser  ──→  AST  ──→  Resolver  ──→  Type Checker  ──→  Codegen
                     │                                                   │
                     │                                                   ├── Android (Kotlin/Compose)
                  Component Registry                                     └── iOS (UIKit/Swift)
                  (55+ components)                                          │
                     │                                                   Validation Files
                  Plugin Registry                                     (native validators)
                     │
                  Stdlib Translator
```

1. **Parser** — PEG grammar produces a typed AST
2. **Resolver** — Validates imports, resolves component references
3. **Type Checker** — Enforces strict type system, validates props/styles against registry
4. **Codegen** — Emits platform-native code (Kotlin + XML for Android, Swift + Storyboard for iOS)
5. **Compiler** — Wraps all phases into `compile()`, returns output files

---

## Contributing

1. Fork the repo
2. Make your changes
3. Run `cargo test` (all tests must pass)
4. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## License

MIT

---

## Plugin System

Plugins extend Frame with native capabilities. Each plugin lives in `frame_modules/<name>/` and contains:

- `plugin.json` — manifest (name, version, permissions, param schemas)
- `src/index.fr` — `.fr` API bridge functions
- `android/<Class>.kt` — native Android implementation
- `ios/<Class>.swift` — native iOS implementation

### Built-in Plugins

#### `frame_camera`

Captures photos via the native camera.

```fr
import { capture } "frame-camera"

fn onCapture: async () => {
    // format: "jpg" | "png" | "webp"  (default "jpg")
    // quality: 0.0–1.0                (default 0.8)
    // source: "camera" | "gallery"    (default "camera")
    :var photo = wait:capture("jpg", 0.9, "camera")
}
```

All params are validated at runtime — invalid values return an error result, not a crash.

#### `frame_storage`

Saves and loads files to local storage.

```fr
import { saveFile, loadFile, deleteFile } "frame-storage"

fn onSave: async () => {
    // directory: "documents" | "cache" | "temp"  (default "documents")
    // encoding:  "utf8" | "base64"               (default "utf8")
    wait:saveFile("notes.txt", "hello world", "documents", "utf8")
    :var content = wait:loadFile("notes.txt", "documents", "utf8")
    wait:deleteFile("notes.txt", "documents")
}
```

Filenames are validated — empty names and path separators (`/`, `\`) are rejected to prevent directory traversal.

#### `frame_connectivity`

Monitors network connectivity state.

```fr
import { isOnline, onNetworkChange } "frame-connectivity"

fn checkNetwork: async () => {
    // type: "any" | "wifi" | "cellular"  (default "any")
    :var online = wait:isOnline("wifi")
    // interval: 1–60 seconds             (default 5, for onNetworkChange)
    wait:onNetworkChange("any", 10)
}
```

### Creating a Plugin

```bash
frame plugin create my-plugin
```

Scaffolds `frame_modules/my-plugin/` with:

```
plugin.json          # manifest + params schema
src/index.fr         # .fr bridge functions
android/MyPlugin.kt  # Android implementation stub
ios/MyPlugin.swift   # iOS implementation stub
README.md
```

### Installing Plugins

```bash
frame plugin add my-plugin          # local scaffold
frame plugin add @user/repo         # GitHub source
frame plugin add @user/repo@v1.2.3  # specific tag
frame plugin list                   # list installed
frame plugin remove my-plugin       # uninstall
```

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `frame start <name>` | Scaffold a new project (MVC or Clean Architecture) |
| `frame start <name> --arch mvc` | Scaffold with MVC architecture |
| `frame start <name> --arch clean` | Scaffold with Clean Architecture |
| `frame build` | Compile all `.fr` files |
| `frame check` | Verify build environment + config |
| `frame test` | Run all `*.test.fr` test suites |
| `frame deploy android` | Generate + build Android project |
| `frame deploy ios` | Generate + build iOS project |
| `frame preview` | Hot-reload development server |
| `frame plugin create <name>` | Create a new plugin scaffold |
| `frame plugin add <name>` | Install a plugin |
| `frame plugin add @user/repo` | Install from GitHub |
| `frame plugin remove <name>` | Remove a plugin |
| `frame plugin list` | List installed plugins |
| `frame plugin install` | Install all plugins from `frame.config.json` |
| `frame init-examples` | Regenerate example projects |

---

## Architecture

```
.fr source files
       │
       ▼
  PEG Parser (grammar.pest)
       │  → AST (ast.rs)
       ▼
  Resolver (resolver/mod.rs)
       │  → import resolution, circular dependency detection
       ▼
  Type Checker (resolver/types.rs)
       │  → :var types, async/await enforcement, prop types
       ▼
  Component Registry (compiler/registry.rs)
       │  → validates built-in component props, children, events
       ▼
  Codegen
    ├── Android (compiler/android.rs)
    │     → Kotlin / Jetpack Compose
    │     → NavHost + typed composable routes
    │     → ViewModel stores with StateFlow + persistence
    │     → ProcessLifecycleOwner for app foreground/background
    └── iOS (compiler/ios.rs)
          → UIKit / UINavigationController
          → Typed ViewController init params
          → FrameRoutable protocol for navigate_back_to
          → NotificationCenter for foreground/background
          → FrameStoreRegistry for store restore-on-launch
```

---

## What's New

### Navigation (Latest)

| Feature | Description |
|---------|-------------|
| `navigate(route, replace: true)` | Replace current back-stack entry |
| `navigate(route, clear_stack: true)` | Clear entire stack before navigating |
| `navigate(route, single_top: true)` | Prevent duplicate screen entries |
| `navigate(route, transition: "slide_up")` | Per-call transition animation hint |
| `navigate_replace(route)` | Shorthand for replace navigation |
| `navigate_back()` | Pop one entry (was already supported) |
| `navigate_back_to(route)` | Pop to a specific named route in the stack |
| `navigate_modal(route)` | Present route as a modal (sheet / dialog) |
| `navigate_dismiss()` | Dismiss current modal |
| `page { params: { id: string } }` | Typed route params — generates typed function/init signatures |
| Tab stacks | `runtime/navigation.rs` supports independent per-tab back stacks |
| `FrameRoutable` protocol (iOS) | All ViewControllers adopt this for `navigate_back_to` |

### App Lifecycle

| Feature | Description |
|---------|-------------|
| `:app {}` top-level block | Declare `on_launch`, `on_foreground`, `on_background` hooks |
| `page { on_mount }` | Called when page is fully visible (viewDidAppear / LaunchedEffect "mount") |
| `page { on_unmount }` | Called when page is fully gone |
| `page { on_foreground }` | Per-page foreground resume hook |
| `page { on_background }` | Per-page background pause hook |
| `before_enter` / `before_leave` | Now accept full expressions (wait: calls, lambdas) not just string names |
| Component `on_mount` | Emitted as `LaunchedEffect(Unit)` / `DispatchQueue.main.async` |
| Component `on_update` + `watch` | `LaunchedEffect(key)` fires when `watch` dependency changes |
| Component `on_unmount` | `DisposableEffect/onDispose` on Android |
| `FrameStoreRegistry` (iOS) | Stores self-register; `AppDelegate` calls `restoreAll()` on launch |
| `ProcessLifecycleOwner` (Android) | App foreground/background observed via `DefaultLifecycleObserver` |
| `SceneDelegate` (iOS) | `sceneWillEnterForeground`, `sceneDidEnterBackground`, `sceneDidBecomeActive`, `sceneWillResignActive` all wired |

### Plugin Architecture

| Feature | Description |
|---------|-------------|
| `plugin.json "params"` schema | Machine-readable param contracts with allowed values and defaults |
| Camera: `format`, `quality`, `source` | All caller-supplied, validated at runtime |
| Storage: `directory`, `encoding` | Caller-supplied; filename validated against path traversal |
| Connectivity: `type`, `interval` | `type` filters by wifi/cellular; `interval` throttles change callbacks |
| `frame_connectivity` plugin | New — monitors network state via `ConnectivityManager` / `NWPathMonitor` |

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make changes following the pipeline: grammar → AST → parser → resolver → codegen → tests
4. Run tests: `cargo test`
5. Open a PR with a clear description

---

## License

MIT
