# Frame

**Frame** is a cross-platform mobile framework with a strictly-typed declarative language (`.fr`). Write UI once — compile to native Kotlin (Android) and Swift (iOS).

- **Declarative**: Describe what your UI looks like, not how to build it
- **Strictly-typed**: Catch errors at compile time, not runtime
- **65+ built-in components**: Layout, input, navigation, media, feedback, gestures — everything you need
- **Native**: Compiles to real Kotlin/Compose and UIKit/Swift — no runtime overhead, no WebView
- **Plugin system**: Extend with community or private plugins
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
  - [Enums & Type Aliases](#enums--type-aliases)
  - [Imports](#imports)
  - [Store / State Management](#store--state-management)
  - [Standard Library](#standard-library)
  - [Logging](#logging)
  - [Events & Lifecycle](#events--lifecycle)
  - [Navigation](#navigation)
  - [App Lifecycle](#app-lifecycle)
  - [Conditionals & Loops](#conditionals--loops)
  - [Animations](#animations)
  - [Testing](#testing)
- [Component Reference](#component-reference)
  - [Layout Components](#layout-components)
  - [Text & Content Components](#text--content-components)
  - [Input Components](#input-components)
  - [Navigation Components](#navigation-components)
  - [Feedback Components](#feedback-components)
  - [Media Components](#media-components)
  - [Gesture Components](#gesture-components)
  - [Plugin Component](#plugin-component)
- [Icon System](#icon-system)
- [Plugin System](#plugin-system)
- [CLI Reference](#cli-reference)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

---

## Quick Start

### Installation

```bash
# Clone
git clone https://github.com/frame-lang/frame.git
cd frame

# Build
cargo build --release

# Add to PATH
ln -sf "$(pwd)/target/release/frame" ~/.local/bin/frame
export PATH="$HOME/.local/bin:$PATH"

# Verify
frame --version
```

### Create and run your first app

```bash
# Scaffold
frame start myApp
cd myApp

# Check environment
frame check

# Build
frame build

# Test
frame test

# Deploy to iOS (macOS with Xcode)
frame deploy ios

# Deploy to Android (Android SDK required)
frame deploy android

# Hot-reload
frame preview
```

### Try the example apps

```bash
frame init-examples
cd examples/blog-app && frame build && frame test
cd ../profile && frame build && frame test
```

---

## Language Guide

### Project Structure

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

### Pages

Pages are the entry points of your app. Each page has a `name`, `route`, optional lifecycle hooks, typed params, and a children tree.

```fr
page: {
    name: "Splash"
    route: "/"
    before_enter: checkAuth
    on_mount: logAppOpen
    styles: { width: 100%  height: 100%  background: $primary }
    children: [
        scaffold: {
            styles: { safe_area: true }
            children: [
                app_bar: { title: "My App"  leading: "line.3.horizontal" }
                column: {
                    styles: { width: 100%  padding: 32  align: "center"  justify: "center" }
                    children: [
                        text: { content: "Welcome!"  styles: { font_size: 32sp  font_weight: "bold"  color: "#FFFFFF" } }
                        spacer: { styles: { height: 16 } }
                        button: { content: "Get Started"  on_click: navigate("/home", clear_stack: true) }
                    ]
                }
            ]
        }
    ]
}
```

```fr
page: {
    name: "Profile"
    route: "/profile/:userId"
    params: { userId: string }
    before_enter: checkAuth
    on_mount: loadProfile
    before_leave: saveEdits
    styles: { safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: { title: "Profile"  leading: "chevron.left" }
                column: {
                    styles: { padding: 16  gap: 12 }
                    children: [
                        avatar: { src: "https://i.pravatar.cc/80" }
                        text: { content: userId }
                        button: { content: "Go Home"  on_click: navigate("/home", clear_stack: true) }
                    ]
                }
            ]
        }
    ]
}
```

### Components

Components are reusable building blocks with typed `props`, `styles`, and `children`.

```fr
import { text, column } "frame-core"

component UserCard: {
    props: {
        name:  string = ""
        email: string = ""
        bio:   string = ""
        avatar_url: string = ""
    }
    styles: {
        border_radius: 8dp
        padding: 12dp
        margin_bottom: 8dp
        background: "#FFFFFF"
    }
    children: [
        column: {
            styles: { gap: 4dp }
            children: [
                avatar: { src: avatar_url  styles: { width: 48  height: 48  border_radius: 24 } }
                text: { content: name  styles: { font_size: 16sp  font_weight: "bold" } }
                text: { content: email  styles: { font_size: 14sp  color: "#666" } }
                text: { content: bio  styles: { font_size: 14sp  font_style: "italic" } }
            ]
        }
    ]
}
```

**Using a component:**

```fr
UserCard: {
    name: UserStore.user_name
    email: UserStore.user_email
    bio: UserStore.user_bio
    avatar_url: UserStore.user_avatar
    show_if: UserStore.user_name != ""
}
```

**Prop rules:**
- `name: type` — required prop (no default, must be passed)
- `name: type = value` — optional with default value
- Access props directly by name inside the component: `name`, `email`, etc.
- `show_if: expr` — conditionally render any component

### Styles

Every component supports a `styles:` block. Over 30 style properties:

```fr
container: {
    styles: {
        width: 100%
        height: 200
        background: "#F5F5F5"
        border_radius: 12dp
        padding: 16dp
        margin_top: 8dp
        overflow: hidden
        safe_area: true
        opacity: 0.9
    }
}
```

| Category       | Properties |
|----------------|------------|
| **Layout**     | `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height`, `flex`, `direction`, `align`, `justify`, `gap`, `aspect_ratio` |
| **Spacing**    | `margin`, `margin_top`, `margin_bottom`, `margin_left`, `margin_right`, `padding`, `padding_top`, `padding_bottom`, `padding_left`, `padding_right` |
| **Appearance** | `background`, `color`, `font_size`, `font_weight`, `font_family`, `border`, `border_radius`, `opacity` |
| **Safe Area**  | `safe_area: true/false` (defaults to `true`) |
| **Overflow**   | `overflow: hidden/scroll/visible`, `overflow_x`, `overflow_y`, `clip_behavior: anti_aliased/hard/none` |
| **Text**       | `text_overflow: ellipsis/clip/fade`, `max_lines`, `line_clamp` |
| **Image**      | `fit: cover/contain/fill/fit_width/fit_height/none` |
| **Scroll**     | `scroll_indicator`, `scroll_snap: start/center/end/none`, `scroll_enabled` |
| **Events**     | `on_scroll`, `on_scroll_end` |

**Responsive breakpoints:**

```fr
:breakpoints { sm: 360dp  md: 600dp  lg: 900dp  xl: 1200dp }

column: {
    styles: {
        width: [100%, @md: 75%, @lg: 50%]
        font_size: [14sp, @md: 16sp, @lg: 18sp]
    }
}
```

### Variables

```fr
:var greeting = "Hello"          // inferred string, immutable
:var mut count = 0               // inferred int, mutable
:var name: string = "World"      // explicit type
:var mut items: list = []        // mutable list

fn process: () => {
    :var count: int = 0
    :var mut total: float = 0.0
    total = total + 1.5           // ok — mutable
}
```

**Theme variables** — defined in `:vars` block at the top of `project.fr`:

```fr
:vars {
    primary:   "#007BFF"
    secondary: "#6C757D"
    success:   "#28A745"
    danger:    "#DC3545"
    bg:        "#F8F9FA"
    radius:    "8dp"
}

// Use with $ prefix anywhere a value is expected
column: {
    styles: { background: $bg  padding: $radius  gap: 8dp }
}
```

### Functions

```fr
// Sync function
fn greet: (name: string) => {
    log.info("Hello, \(name)!")
}

// Async function
fn loadUser: async (id: string) => {
    UserStore.is_loading = true
    try {
        result = wait:fetch("/api/users/$id", { method: "GET" })
        if result != null {
            UserStore.user = result
        }
    } catch (err) {
        UserStore.error = err
    }
    UserStore.is_loading = false
}

// Function with default params
fn fetchData: async (url: string, method: string = "GET") => {
    return wait:fetch(url, { method: method })
}

// Function with return value
fn double: (x: int) => {
    return x * 2
}

// Calling
fn process: async () => {
    wait:loadUser("1")
    :var data = wait:fetchData("/api/items")
    :var result = double(5)
}
```

**Rules:**
- Async functions must be called with `wait:` prefix
- Calling an async function without `wait:` is a compile error
- Parameters can have default values: `name: type = default`
- Use `return expr` to return a value

### String Interpolation

Both `\(expr)` and `${expr}` work inside strings:

```fr
:var name = "Alice"
:var count = 3

text: { content: "Hello \(name), you have \(count) messages" }
text: { content: "Path: ${path}/file.txt" }

// In fetch URLs
result = wait:fetch("/api/users/$id", { method: "GET" })
```

### Named Arguments

```fr
fn greet: (name: string, greeting: string = "Hello") => {
    log.info("\(greeting), \(name)")
}

// Call with named args
greet(name: "Alice")
greet(name: "Bob", greeting: "Hi")

// Wait calls
wait:UserStore.load(id: "1")
```

### Fetch & Headers

```fr
fn loadProfile: async () => {
    try {
        result = wait:fetch("/api/users/1", {
            method: "GET"
            headers: {
                Authorization: "Bearer $token"
                Content-Type: "application/json"
            }
            timeout: 10000
        })

        // Chaining
        result = wait:fetch("/api/data")
            .then((data) => {
                log.info("Got: \(data)")
                return data
            })
            .catch((err) => {
                log.error("Failed: \(err)")
            })

        if result != null {
            UserStore.data = result
        }
    } catch (err) {
        UserStore.error = err
    }
}
```

### Validation

```fr
:validation UserSchema {
    name:  required | min(2) | max(100)
    email: required | email
    age:   optional | min(0) | max(150)
}

// Inline validation
input: {
    value: email
    placeholder: "Email"
    validate: required | email
    on_error: showEmailError()
}

// Form with schema
form: {
    schema: UserSchema
    children: [
        input: { value: formName  placeholder: "Name" }
        input: { value: formEmail  placeholder: "Email" }
        input: { value: formAge  placeholder: "Age" }
    ]
}
```

**Built-in validators:** `required`, `optional`, `email`, `min(n)`, `max(n)`, `min_length(n)`, `max_length(n)`, `pattern(regex)`, `url`.

### Enums & Type Aliases

```fr
:enum Status {
    Active
    Inactive
    Pending
}

:enum Color {
    Red   = "#FF0000"
    Green = "#00FF00"
    Blue  = "#0000FF"
}

:type UserId = string
:type Score  = int
:type Callback = () => void
```

### Imports

```fr
// Built-in components
import { text, button, column, scaffold, app_bar } "frame-core"

// Relative path imports
import { UserCard }  "../components/UserCard.fr"
import { loadUser }  "../../controllers/UserController.fr"

// Plugin imports
import { capture }         "frame-camera"
import { isOnline }        "frame-connectivity"
import { saveFile }        "frame-storage"

// Import with alias
import { UserCard as Card } "../components/UserCard.fr"

// Re-export
export { UserCard } "../components/UserCard.fr"
```

### Store / State Management

```fr
:store UserStore {
    user_name: string = "Jane Smith"
    user_email: string = "jane@example.com"
    user_bio: string = "Full-stack developer"
    is_loading: bool = false
    error: string = ""
    dark_mode: bool = false
    volume: float = 50

    fn load: async (id: string) => {
        UserStore.is_loading = true
        UserStore.error = ""
        // Simulate API call
        wait:sleep(500)
        UserStore.user_name = "Jane Smith"
        UserStore.user_email = "jane@example.com"
        UserStore.user_bio = "Full-stack developer & photographer"
        UserStore.is_loading = false
    }

    fn toggleDarkMode: () => {
        if UserStore.dark_mode == true {
            UserStore.dark_mode = false
        } else {
            UserStore.dark_mode = true
        }
    }

    fn setVolume: (val: float) => {
        UserStore.volume = val
    }
}
```

**Reading store state in UI:**

```fr
text: {
    content: "Loading..."
    show_if: UserStore.is_loading
}
text: {
    content: UserStore.user_name
    styles: { font_size: 18sp  font_weight: "bold" }
}
text: {
    content: UserStore.error
    styles: { color: "#FF0000" }
    show_if: UserStore.error != ""
}
switch: {
    value: UserStore.dark_mode
    on_change: wait:UserStore.toggleDarkMode()
}
slider: {
    value: UserStore.volume
    min: 0  max: 100
    on_change: wait:UserStore.setVolume()
}
```

**Persistent fields:**

```fr
:store AuthStore {
    token: string = ""
    theme: string = "light"
    persist: {
        token: secure    // Keychain (iOS) / EncryptedSharedPreferences (Android)
        theme: local     // UserDefaults (iOS) / SharedPreferences (Android)
    }
}
```

**Store field types:** `int`, `float`, `bool`, `string`, `object`, `list`

### Standard Library

#### String Methods

```fr
string.upper("hello")          // "HELLO"
string.lower("HELLO")          // "hello"
string.trim("  hi  ")          // "hi"
string.contains("hello", "el") // true
string.starts_with("hello", "he") // true
string.ends_with("hello", "lo")   // true
string.replace("a-b-c", "-", "_") // "a_b-c"
string.replace_all("a-b-c", "-", "_") // "a_b_c"
string.split("a,b,c", ",")     // ["a","b","c"]
string.join(["a","b","c"], ",") // "a,b,c"
string.length("hello")         // 5
string.is_empty("")            // true
string.slice("hello", 1, 4)    // "ell"
string.to_int("42")            // 42
string.to_float("3.14")        // 3.14
string.pad_left("5", 3, "0")   // "005"
string.pad_right("5", 3, "0")  // "500"
```

#### Number Functions

```fr
number.abs(-5)                 // 5
number.sqrt(16)                // 4
number.floor(3.7)              // 3
number.ceil(3.2)               // 4
number.round(3.5)              // 4
number.min(3, 7)               // 3
number.max(3, 7)               // 7
number.clamp(15, 0, 10)        // 10
number.random()                // 0.0..1.0
```

#### List Methods

```fr
list.length([1, 2, 3])         // 3
list.contains([1, 2, 3], 2)    // true
list.is_empty([])              // true
list.first([1, 2, 3])          // 1
list.last([1, 2, 3])           // 3
list.reverse([1, 2, 3])        // [3, 2, 1]
list.sum([1, 2, 3])            // 6
list.average([1, 2, 3])        // 2.0
```

#### Math Functions

```fr
math.abs(-5)                   // 5
math.sqrt(16)                  // 4
math.sin(0)                    // 0.0
math.cos(0)                    // 1.0
math.pi                        // 3.14159...
math.pow(2, 3)                 // 8
math.log(2.718)                // 1.0
```

#### Date Functions

```fr
date.now()                     // current timestamp
date.format(date.now(), "yyyy-MM-dd") // "2026-07-13"
```

#### Object/Map Functions

```fr
object.keys({a: 1, b: 2})      // ["a", "b"]
object.values({a: 1, b: 2})    // [1, 2]
object.has_key({a: 1}, "a")    // true
```

#### JSON

```fr
from_json('{"name":"Alice"}')  // { name: "Alice" }
to_json({name: "Alice"})       // '{"name":"Alice"}'
```

#### Utility

```fr
util.print("hello")            // prints to console
util.type_of(42)               // "int"
util.is_null(null)             // true
util.is_not_null("hello")      // true
util.uuid()                    // "550e8400-e29b-..."
util.encode_base64("hello")    // "aGVsbG8="
util.decode_base64("aGVsbG8=") // "hello"
util.encode_url("hello world") // "hello%20world"
util.decode_url("hello%20world") // "hello world"
```

### Logging

```fr
log.info("App started")
log.warn("Cache miss for user \(userId)")
log.error("Failed to load: \(errorMessage)")
log.debug("Button pressed at \(x), \(y)")
log.verbose("Detailed debug: \(state)")
```

| Level | Android | iOS |
|-------|---------|-----|
| VERBOSE | `Log.v(...)` | `os_log(.debug, ...)` |
| DEBUG | `Log.d(...)` | `os_log(.debug, ...)` |
| INFO | `Log.i(...)` | `os_log(.info, ...)` |
| WARN | `Log.w(...)` | `os_log(.default, ...)` |
| ERROR | `Log.e(...)` | `os_log(.error, ...)` |

### Events & Lifecycle

**Component events:**

```fr
button: {
    content: "Submit"
    on_click: handleSubmit()
}
input: {
    value: email
    on_change: validateEmail()
    on_focus: logFocus()
    on_blur: logBlur()
    on_submit: submitForm()
}
slider: {
    value: volume
    on_change: adjustVolume()
}
switch: {
    value: enabled
    on_change: wait:Store.toggle()
}
```

**Component lifecycle hooks:**

```fr
column: {
    on_mount:   loadInitialData()     // fires once after first render (viewDidAppear)
    on_update:  refreshList()         // fires when watched dependency changes
    watch:      UserStore.items       // dependency for on_update
    on_unmount: cancelRequests()      // fires when node is removed
    children: [ ... ]
}
```

**Page lifecycle hooks:**

```fr
page: {
    name: "Dashboard"
    route: "/dashboard"
    before_enter: checkAuth()        // guard called before transition
    on_mount:     loadData()         // called when fully visible
    before_leave: saveState()        // called when navigating away
    on_unmount:   cleanup()          // called on dispose
    on_background: pausePolling()    // app backgrounded
    on_foreground: resumePolling()   // app foregrounded
}
```

### Navigation

```fr
// Basic navigation
navigate("/profile")
navigate("/profile/\(userId)")

// With options
navigate("/home", clear_stack: true)      // clear entire back stack
navigate("/detail", replace: true)        // replace current screen
navigate("/detail", single_top: true)     // avoid duplicate screens
navigate("/detail", transition: "slide_up")

// Navigation functions
navigate_replace("/login")                // replace, no back entry
navigate_back()                           // pop one screen
navigate_back_to("/home")                 // pop to specific route
navigate_modal("/settings")               // present modally
navigate_dismiss()                        // dismiss modal
```

| Function | Android | iOS |
|----------|---------|-----|
| `navigate(route)` | `navController.navigate(route)` | `pushViewController(routeVC(for:))` |
| `navigate_replace(route)` | `popUpTo(current) { inclusive=true }` | Pop + push |
| `navigate_back()` | `navController.popBackStack()` | `popViewController(animated:)` |
| `navigate_back_to(route)` | `popBackStack(route, false)` | `popToViewController` |
| `navigate_modal(route)` | `navController.navigate(route)` | `present(routeVC(for:))` |
| `navigate_dismiss()` | `popBackStack()` | `dismiss(animated:)` |

### App Lifecycle

Declared once in `project.fr`:

```fr
:app {
    on_launch:     appInit          // Application.onCreate / didFinishLaunching
    on_foreground: appForeground    // ON_START / willEnterForeground
    on_background: appBackground    // ON_STOP / didEnterBackground
}

fn appInit: () => {
    log.info("App launched")
}
fn appForeground: () => {
    log.info("App foregrounded")
}
fn appBackground: () => {
    log.info("App backgrounded")
}
```

### Conditionals & Loops

**Conditional rendering with `show_if`:**

```fr
text: { content: "Loading..."  show_if: UserStore.is_loading }
skeleton: { show_if: UserStore.is_loading }
UserCard: { name: UserStore.user_name  show_if: UserStore.user_name != "" }
text: { content: UserStore.error  styles: { color: "#FF0000" }  show_if: UserStore.error != "" }
```

**if/else in functions:**

```fr
fn checkAccess: async () => {
    :var online = wait:isOnline("any")
    if online != true {
        navigate_modal("/offline")
    } else {
        wait:loadInitialData()
    }
}
```

**for loops:**

```fr
fn processItems: (items: list) => {
    for item in items {
        log.debug("Processing: \(item)")
    }
}
```

**switch/case:**

```fr
fn handleStatus: (status: string) => {
    switch status {
        case "active" => { log.info("Active user") }
        case "inactive" => { log.warn("Inactive user") }
        case "banned" => { log.error("Banned user") }
    }
}
```

**Data-bound list:**

```fr
list: {
    data: UserStore.users
    build: (user) => {
        UserCard: { name: user.name  email: user.email }
    }
}
```

### Animations

```fr
button: {
    content: "Animated"
    animate: {
        property: opacity
        from: 0
        to: 1
        duration: 300ms
        delay: 100ms
        easing: ease_in_out
        repeat: 0
        auto_reverse: false
    }
}
```

| Property | Values |
|----------|--------|
| `property` | `opacity`, `scale_x`, `scale_y`, `rotation`, `translation_x`, `translation_y` |
| `easing` | `ease_in_out`, `ease_in`, `ease_out`, `linear`, `bounce`, `spring` |
| `duration` | Time in ms (e.g. `300ms`, `1s`) |

### Testing

```fr
describe: "UserStore" => {
    it: "is_loading starts false" => {
        expect: false .toBeFalse:()
    }
    it: "error starts empty" => {
        expect: "" .toBe: ""
    }
}

describe: "API" => {
    it: "fetches user data" => {
        mock: {
            url: "/api/users/1"
            response: { id: "1"  name: "Jane"  email: "jane@test.com" }
            status: 200
        }
        expect: "Jane" .toBe: "Jane"
    }
    it: "handles 404 gracefully" => {
        mock: {
            url: "/api/users/999"
            response: { error: "Not found" }
            status: 404
        }
        expect: "Not found" .toBe: "Not found"
    }
}
```

**Matchers:** `.toBe: value`, `.toEqual: value`, `.toContain: value`, `.toBeNull:()`, `.toBeTrue:()`, `.toBeFalse:()`, `.toThrow:()`

**Run:** `frame test`

---

## Component Reference

Each component includes a real code snippet showing required and optional props, styles, events, and children.

---

### Layout Components

#### `row`

Horizontally arranges children in a flex row.

```fr
row: {
    styles: { gap: 8dp  padding: 16dp  justify: "space_between"  align: "center" }
    children: [
        text: { content: "Left" }
        text: { content: "Center" }
        text: { content: "Right" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events |
|--------|--------|
| All layout styles | `on_click`, `on_scroll`, `on_scroll_end` |

| Platform | Mapping |
|----------|---------|
| iOS | `UIStackView` axis = `.horizontal` |
| Android | `Row` composable |

#### `column`

Vertically arranges children in a flex column. The most common layout component.

```fr
column: {
    styles: {
        width: 100%
        height: 100%
        padding: 16dp
        gap: 12dp
        overflow: scroll
    }
    children: [
        text: { content: "Title"  styles: { font_size: 18sp  font_weight: "bold" } }
        text: { content: "Body content goes here"  styles: { font_size: 14sp  color: "#666" } }
        button: { content: "Action"  on_click: handleAction() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events |
|--------|--------|
| All layout styles | `on_click`, `on_scroll`, `on_scroll_end` |

| Platform | Mapping |
|----------|---------|
| iOS | `UIStackView` axis = `.vertical` |
| Android | `Column` composable |

#### `container`

Generic box container — a `UIView` / `Box` with no layout direction.

```fr
container: {
    styles: {
        background: "#F5F5F5"
        border_radius: 12dp
        padding: 16dp
        overflow: hidden
        width: 200
        height: 200
    }
    children: [
        text: { content: "Inside box" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events |
|--------|--------|
| All layout styles | `on_click` |

#### `stack`

Positions children in z-order (layered on top of each other). Use `alignment` for child positioning.

```fr
stack: {
    alignment: center
    children: [
        image: { src: "https://example.com/bg.jpg"  styles: { width: 100%  height: 200dp } }
        text: {
            content: "Overlay"
            styles: { color: "#FFFFFF"  font_size: 18sp  font_weight: "bold" }
        }
    ]
}
```

**Positioned children:**

```fr
stack: {
    styles: { width: 300  height: 300 }
    children: [
        text: { content: "Centered" }
        text: {
            content: "Top Left"
            positioned: { top: 8  left: 8 }
        }
        text: {
            content: "Bottom Right"
            positioned: { bottom: 8  right: 8  width: 120  height: 40 }
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `alignment` | String | No | — | `top_left`, `top_center`, `top_right`, `center_left`, `center`, `center_right`, `bottom_left`, `bottom_center`, `bottom_right` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any, can use `positioned:{}`) |

#### `scaffold`

Top-level screen structure. Handles safe area, app bar, and bottom navigation. Designed to wrap page content.

```fr
scaffold: {
    styles: { safe_area: true }
    children: [
        app_bar: { title: "My App"  leading: "line.3.horizontal" }
        column: {
            styles: { padding: 16dp }
            children: [ text: { content: "Content here" } ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | First child `app_bar` → TopAppBar slot, first `bottom_navigation_bar` → BottomAppBar slot |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `card`

Elevated container with platform shadow.

```fr
card: {
    styles: { padding: 16dp  margin_top: 8dp  border_radius: 12dp }
    children: [
        text: { content: "Card Title"  styles: { font_size: 18sp  font_weight: "bold" } }
        spacer: { styles: { height: 8dp } }
        text: { content: "Card body with supporting text."  styles: { font_size: 14sp  color: "#666" } }
        button: { content: "Learn More"  on_click: openDetail() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

#### `divider`

Thin horizontal separator line.

```fr
divider: {}
divider: { styles: { color: "#E0E0E0"  margin: 8dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| `color`, `margin` | — |

#### `spacer`

Invisible space used for flexible gaps.

```fr
spacer: { styles: { height: 16dp } }
spacer: { styles: { width: 8dp } }
spacer: { styles: { width: 100%  height: 20dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| `width`, `height` | — |

#### `scroll_view`

Scrollable wrapper. Contents can overflow and scroll.

```fr
scroll_view: {
    styles: { width: 100%  height: 100% }
    children: [
        column: {
            styles: { gap: 12dp  padding: 16dp }
            children: [
                text: { content: "Item 1" }
                text: { content: "Item 2" }
                text: { content: "Item 3 (scroll to see)" }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events |
|--------|--------|
| All layout styles | `on_scroll`, `on_scroll_end` |

#### `list`

Data-bound repeating list. Provide `data` (expression) and `build` (item render function).

```fr
list: {
    data: UserStore.items
    build: (item) => {
        text: { content: item.name  styles: { padding: 8dp } }
    }
}

list: {
    data: UserStore.users
    build: (user) => {
        UserCard: { name: user.name  email: user.email  bio: user.bio }
    }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `data` | Expression | No | — | List data source |
| `build` | Expression | No | — | Render function `(item) => { ... }` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_scroll`, `on_scroll_end` | Yes (fallback when no data/build) |

#### `grid`

Grid layout with fixed columns.

```fr
grid: {
    columns: 2
    styles: { gap: 8dp  padding: 16dp }
    children: [
        card: { children: [ text: { content: "Item 1" } ] }
        card: { children: [ text: { content: "Item 2" } ] }
        card: { children: [ text: { content: "Item 3" } ] }
        card: { children: [ text: { content: "Item 4" } ] }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `columns` | Int | No | — | Number of columns |
| `data` | Expression | No | — | Data source (alternative to children) |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any). Falls back to children if no data prop |

#### `form`

Form container with validation schema.

```fr
form: {
    schema: UserSchema
    children: [
        input: { value: formName  placeholder: "Name" }
        input: { value: formEmail  placeholder: "Email" }
        button: { content: "Submit"  on_click: submitForm() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `schema` | String | No | — | `:validation` block name to apply |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_submit` | Yes (any) |

#### `accordion`

Expandable/collapsible section with a title header.

```fr
accordion: {
    title: "More Details"
    styles: { padding: 12dp  border_radius: 8dp }
    children: [
        text: { content: "Hidden content revealed on tap"  styles: { padding: 8dp } }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | String | No | — | Header text |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `timeline`

Vertical timeline display. Each child becomes a timeline item.

```fr
timeline: {
    styles: { padding: 16dp }
    children: [
        text: { content: "Step 1: Created"  styles: { font_size: 14sp } }
        text: { content: "Step 2: In Review"  styles: { font_size: 14sp } }
        text: { content: "Step 3: Approved"  styles: { font_size: 14sp } }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `item`

Generic list item wrapper. Used inside `list` or `grid` children.

```fr
item: {
    styles: { padding: 12dp  border_radius: 8dp  background: "#FFF" }
    children: [
        text: { content: "List item" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `plugin`

Custom plugin component bridge. See [Plugin Component](#plugin-component).

---

### Text & Content Components

#### `text`

Display formatted text. Supports rich styling and string interpolation.

```fr
text: { content: "Hello, World!" }

text: {
    content: "Hello \(name), you have \(count) messages"
    styles: {
        font_size: 18sp
        font_weight: "bold"
        color: "#333333"
        text_overflow: ellipsis
        max_lines: 2
        font_family: "Helvetica"
    }
    on_click: handleTap()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | `""` | Text content |

| Styles | Events | Children |
|--------|--------|----------|
| `color`, `font_size`, `font_weight`, `font_family`, `text_overflow`, `max_lines`, `line_clamp`, `width`, `height`, `margin`, `padding`, `opacity` | `on_click` | No |

#### `button`

Tappable button. Styled with layout properties.

```fr
button: { content: "Submit" }

button: {
    content: "Get Started"
    styles: {
        background: "#007BFF"
        color: "#FFFFFF"
        border_radius: 8dp
        padding: 12dp 24dp
        margin_top: 16dp
    }
    on_click: navigate("/home", clear_stack: true)
}

button: {
    content: "Delete"
    styles: { background: "#DC3545"  color: "#FFFFFF" }
    on_click: confirmDelete()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | `""` | Button label |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | No |

#### `icon`

Displays an icon. Uses SF Symbols on iOS, Material Icons on Android, or custom SVG path data.

```fr
// SF Symbol / Material Icon name
icon: { name: "heart"  styles: { color: "#FF0000"  width: 24dp  height: 24dp } }

// Custom SVG path
icon: { path: "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5..."  styles: { width: 24  height: 24 } }

// With click handler
icon: {
    name: "gearshape"
    styles: { color: "#333"  width: 24  height: 24 }
    on_click: openSettings()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `name` | String | No | — | Platform icon name (SF Symbol or Material) |
| `path` | String | No | — | Custom SVG path data |

| Styles | Events | Children |
|--------|--------|----------|
| `color`, `font_weight`, `width`, `height`, `opacity`, `margin` | `on_click` | No |

#### `image`

Displays an image from URL or local asset.

```fr
image: { src: "https://example.com/photo.jpg" }

image: {
    src: "https://example.com/photo.jpg"
    styles: {
        width: 100%
        height: 200dp
        fit: cover
        border_radius: 12dp
    }
    on_click: enlarge()
}

image: {
    src: "assets/logo.png"
    styles: { width: 120dp  height: 120dp  fit: contain }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Image URL or asset path |

| Styles | Events | Children |
|--------|--------|----------|
| `width`, `height`, `border_radius`, `opacity`, `margin`, `fit`, `clip_behavior` | `on_click` | No |

`fit` values: `cover`, `contain`, `fill`, `fit_width`, `fit_height`, `none`

#### `avatar`

Circular avatar image. Automatically clips to circle.

```fr
avatar: { src: "https://i.pravatar.cc/80" }

avatar: {
    src: UserStore.user_avatar
    styles: { width: 48dp  height: 48dp }
    on_click: viewProfile()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Avatar image URL |

| Styles | Events | Children |
|--------|--------|----------|
| `width`, `height`, `border_radius`, `opacity`, `margin`, `fit`, `clip_behavior` | `on_click` | No |

#### `badge`

Notification badge showing a count.

```fr
badge: { count: 5 }

column: {
    children: [
        icon: { name: "bell" }
        badge: { count: UserStore.unread_count }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `count` | Int | No | — | Badge number |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `chip`

Compact interactive chip/tag element.

```fr
chip: { content: "React" }

chip: {
    content: "Filter"
    styles: { background: "#E3F2FD"  border_radius: 16dp  padding: 8dp 16dp }
    on_click: applyFilter()
}

chip: { content: UserStore.selected_tag  label: "Tag" }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Chip text |
| `label` | String | No | — | Optional label |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | No |

#### `tag`

Visual label/tag. Non-interactive (no click event).

```fr
tag: { content: "New" }

tag: {
    content: "Beta"
    styles: { background: "#FFF3CD"  color: "#856404"  border_radius: 4dp  padding: 4dp 8dp }
}

tag: { content: "Important"  label: "Priority" }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Tag text |
| `label` | String | No | — | Optional label |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `banner`

Prominent banner with background color.

```fr
banner: {
    styles: { background: "#E3F2FD"  padding: 12dp  border_radius: 8dp }
    children: [
        text: { content: "New version available!"  styles: { font_weight: "bold" } }
    ]
    on_click: openUpdate()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

#### `skeleton`

Loading placeholder that shows a gray shimmer animation.

```fr
skeleton: { show_if: UserStore.is_loading }

skeleton: {
    show_if: data == null
    styles: { width: 100%  height: 80dp  border_radius: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| All layout styles | — |

#### `table`

Tabular data display.

```fr
table: {
    data: UserStore.rows
    styles: { width: 100%  padding: 8dp }
    children: [
        row: {
            children: [
                text: { content: "Name"  styles: { font_weight: "bold" } }
                text: { content: "Email"  styles: { font_weight: "bold" } }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `data` | Expression | No | — | Table data source |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

---

### Input Components

#### `input`

Single-line text input field.

```fr
input: { value: searchQuery  placeholder: "Search..." }

input: {
    value: email
    placeholder: "Enter your email"
    validate: required | email
    on_error: showError()
    on_change: validateEmail()
    on_submit: submitForm()
    on_focus: logFocus()
    on_blur: logBlur()
    styles: {
        padding: 12dp
        border_radius: 8dp
        border: "1px solid #CCC"
        width: 100%
    }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Current input value |
| `placeholder` | String | No | — | Placeholder text |
| `validate` | Expression | No | — | Validation rules |
| `on_error` | String | No | — | Error handler |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_submit`, `on_focus`, `on_blur` | No |

#### `text_area`

Multi-line text input.

```fr
text_area: { value: bio  placeholder: "Tell us about yourself..." }

text_area: {
    value: description
    placeholder: "Enter description"
    lines: 5
    validate: required | min_length(10)
    on_change: validateDescription()
    styles: { width: 100%  height: 120dp  padding: 12dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Current text |
| `placeholder` | String | No | — | Placeholder |
| `lines` | Int | No | — | Number of visible lines |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_submit` | No |

#### `dropdown`

Dropdown / select menu. Children are the options.

```fr
dropdown: {
    value: selectedOption
    validate: required
    on_change: handleSelect()
    children: [
        text: { content: "Option A" }
        text: { content: "Option B" }
        text: { content: "Option C" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Currently selected value |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_select` | Yes (any — rendered as options) |

#### `switch`

Toggle switch. Uses `value` for checked state (synonym: `checked`).

```fr
switch: { value: notificationsEnabled }

switch: {
    value: UserStore.notifications_enabled
    on_change: wait:UserStore.toggleNotifications()
}

switch: { checked: darkMode  on_change: toggleDarkMode() }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Bool | No | — | Checked state |
| `checked` | Bool | No | — | Synonym for `value` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `checkbox`

Checkbox input.

```fr
checkbox: { value: agreeToTerms }

checkbox: {
    value: UserStore.opted_in
    on_change: wait:UserStore.setOptIn()
}

checkbox: { checked: isSelected  on_change: toggleSelect() }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Bool | No | — | Checked state |
| `checked` | Bool | No | — | Synonym for `value` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `radio`

Radio button for single-selection groups.

```fr
radio: { selected: isChosen }

radio: {
    selected: UserStore.selected == "option_a"
    on_click: wait:UserStore.selectOption("option_a")
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `selected` | Bool | No | — | Whether this radio is selected |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | No |

#### `slider`

Range slider for numeric values.

```fr
slider: { value: volume }

slider: {
    value: UserStore.volume
    min: 0
    max: 100
    on_change: wait:UserStore.setVolume()
    styles: { width: 80%  padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Float | No | — | Current value |
| `min` | Float | No | — | Minimum value |
| `max` | Float | No | — | Maximum value |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `stepper`

Increment/decrement stepper with + and - buttons.

```fr
stepper: { value: quantity }

stepper: {
    value: UserStore.item_count
    on_increment: wait:UserStore.increment()
    on_decrement: wait:UserStore.decrement()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Int | No | — | Current value |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_increment`, `on_decrement` | No |

#### `search_bar`

Search input with magnifying glass icon and clear button.

```fr
search_bar: { value: query  placeholder: "Search..." }

search_bar: {
    value: UserStore.search_query
    placeholder: "Search users..."
    on_change: wait:UserStore.search()
    on_submit: executeSearch()
    styles: { width: 100%  padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Search query |
| `placeholder` | String | No | — | Placeholder text |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change`, `on_submit` | No |

#### `date_picker`

Date picker. On iOS uses inline date picker, on Android uses Material DatePicker.

```fr
date_picker: { value: selectedDate }

date_picker: {
    value: UserStore.birth_date
    validate: required
    on_change: wait:UserStore.setDate()
    styles: { padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Date string |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `time_picker`

Time picker. On iOS uses wheels style, on Android uses Material TimeInput.

```fr
time_picker: { value: selectedTime }

time_picker: {
    value: UserStore.reminder_time
    validate: required
    on_change: wait:UserStore.setTime()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Time string |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `color_picker`

Color picker. On iOS uses UIColorWell, on Android renders a button showing selected color.

```fr
color_picker: { value: selectedColor }

color_picker: {
    value: UserStore.theme_color
    on_change: wait:UserStore.setColor()
    styles: { padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Hex color string |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `rating`

Star rating display/interaction.

```fr
rating: { value: 3 }

rating: {
    value: UserStore.rating
    max: 5
    on_change: wait:UserStore.setRating()
}

rating: {
    value: 4
    max: 10
    styles: { padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Int | No | — | Current rating |
| `max` | Int | No | `5` | Maximum stars |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_change` | No |

#### `otp_input`

One-time password input. Shows individual digit fields.

```fr
otp_input: { length: 6 }

otp_input: {
    length: 4
    validate: required | length(4)
    on_complete: verifyOTP()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `length` | Int | No | `6` | Number of digits |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

---

### Navigation Components

#### `app_bar`

Top app bar with title, leading icon, and trailing action items.

```fr
app_bar: { title: "Home" }

app_bar: {
    title: "Frame App"
    leading: "line.3.horizontal"
    children: [
        icon: { name: "magnifyingglass"  on_click: openSearch() }
        icon: { name: "gearshape"  on_click: openSettings() }
    ]
}

app_bar: {
    title: "Profile"
    leading: "chevron.left"
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | String | No | — | Title text |
| `leading` | String | No | — | Icon name for nav icon (hamburger, back) |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (icon/button children become trailing actions) |

**Usage:** Place as direct child of `scaffold`. On Android → `TopAppBar` slot. On iOS → `navigationItem.title` + bar button items.

#### `bottom_navigation_bar`

Bottom tab navigation bar. Designed as a scaffold child.

```fr
bottom_navigation_bar: {
    styles: { background: "#FFFFFF" }
    children: [
        tab: { content: "Home"  icon: "house.fill" }
        tab: { content: "Search"  icon: "magnifyingglass" }
        tab: { content: "Profile"  icon: "person.fill" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Children rendered as tab items |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `sidebar`

Side drawer panel. Pins to left or right edge.

```fr
row: {
    styles: { width: 100%  height: 100% }
    children: [
        sidebar: {
            side: "left"
            width: "280"
            styles: { background: "#F8F9FA"  padding: 8dp }
            children: [
                text: { content: "Menu"  styles: { font_weight: "bold"  padding: 8dp } }
                button: { content: "Dashboard"  on_click: navigate("/dashboard") }
                button: { content: "Profile"  on_click: navigate("/profile/1") }
                button: { content: "Settings"  on_click: navigate_modal("/settings") }
                divider: {}
                text: { content: "Tags"  styles: { font_weight: "bold"  padding: 8dp } }
                chip: { content: "Important" }
                tag: { content: "New" }
            ]
        }
        column: {
            styles: { padding: 16dp }
            children: [ text: { content: "Main Content" } ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `side` | String | No | `"left"` | `"left"` or `"right"` |
| `width` | String | No | `"260"` | Width in dp |
| `collapsed` | Bool | No | — | Whether collapsed |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any) |

#### `floating_action_button`

Circular action button (FAB). Positioned at bottom-end by default.

```fr
// With icon prop
floating_action_button: {
    icon: "plus"
    on_click: handleAdd()
}

// With child icon component
floating_action_button: {
    children: [
        icon: { name: "plus"  styles: { color: "#FFFFFF"  width: 24  height: 24 } }
    ]
    on_click: handleAdd()
}

// With content label
floating_action_button: {
    content: "Save"
    icon: "checkmark"
    position: "bottom_end"
    on_click: saveData()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Label text |
| `icon` | String | No | — | Icon name |
| `position` | String | No | `"bottom_end"` | `bottom_end`, `bottom_start`, `top_end`, `top_start` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click` | Yes (rendered inside FAB) |

#### `tab_bar`

Horizontal tab bar. Only accepts `tab` children.

```fr
tab_bar: {
    selected: 0
    children: [
        tab: { content: "Chats"  icon: "message.fill"  on_select: switchToChats() }
        tab: { content: "Status"  icon: "circle.fill"  on_select: switchToStatus() }
        tab: { content: "Calls"  icon: "phone.fill"  on_select: switchToCalls() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `selected` | Int | No | — | Selected tab index |
| `current` | Int | No | — | Synonym for `selected` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (only `tab` children) |

#### `tab`

Individual tab item. Used inside `tab_bar`.

```fr
tab: { content: "Home"  icon: "house.fill"  on_click: goHome() }

tab: {
    content: "Settings"
    icon: "gearshape"
    on_select: openSettings()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Tab label |
| `icon` | String | No | — | Tab icon name |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_click`, `on_select` | No |

#### `bottom_sheet`

Modal bottom sheet. Slides up from bottom.

```fr
bottom_sheet: {
    styles: { padding: 16dp }
    children: [
        text: { content: "Sheet Title"  styles: { font_size: 18sp  font_weight: "bold" } }
        text: { content: "Sheet content" }
        button: { content: "Close"  on_click: navigate_dismiss() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_unmount` | Yes (any) |

#### `modal`

Alert-style dialog. For simple confirmations and messages.

```fr
modal: { title: "Confirm"  message: "Are you sure?" }

modal: {
    title: "Delete Item"
    message: "This action cannot be undone."
    on_unmount: handleDismiss()
    children: [
        row: {
            styles: { gap: 8dp  justify: "end" }
            children: [
                button: { content: "Cancel"  on_click: navigate_dismiss() }
                button: { content: "Delete"  styles: { color: "#FF0000" }  on_click: confirmDelete() }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | String | No | — | Dialog title |
| `message` | String | No | — | Dialog message |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_unmount` | Yes (any) |

---

### Feedback Components

#### `toast`

Transient notification that auto-dismisses.

```fr
toast: { message: "Saved successfully!" }

toast: {
    message: "Error saving data"
    duration: 3000
}

button: { content: "Show Toast"  on_click: showToast("Hello from Frame!") }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `message` | String | No | — | Toast message text |
| `duration` | Int | No | — | Display duration in ms |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `tooltip`

Contextual tooltip on hover/long-press.

```fr
tooltip: {
    text: "This is helpful info"
    styles: { padding: 8dp }
    children: [
        text: { content: "Hover over me" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `text` | String | No | — | Tooltip text |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | Yes (any — trigger element) |

#### `progress_bar`

Horizontal progress bar. Value from 0.0 to 1.0.

```fr
progress_bar: { value: 0.65 }

progress_bar: {
    value: UserStore.upload_progress
    styles: { width: 100%  height: 4dp  border_radius: 2dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Float | No | — | Progress 0.0–1.0 |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `progress_circle`

Circular progress indicator.

```fr
progress_circle: { value: 0.8 }

progress_circle: { value: UserStore.loading_progress }

progress_circle: {
    styles: { width: 48dp  height: 48dp  color: "#007BFF" }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Float | No | — | Progress 0.0–1.0 |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

---

### Media Components

#### `video_player`

Video playback using native player (AVPlayer on iOS, ExoPlayer on Android).

```fr
video_player: { src: "https://example.com/video.mp4" }

video_player: {
    src: UserStore.video_url
    on_complete: handleVideoEnd()
    styles: { width: 100%  height: 300dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Video URL |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

#### `audio_player`

Audio playback using native player.

```fr
audio_player: { src: "https://example.com/audio.mp3" }

audio_player: {
    src: UserStore.audio_url
    on_complete: trackFinished()
    styles: { width: 100% }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Audio URL |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

#### `lottie`

Lottie animation. Requires `lottie-ios` pod on iOS and Lottie Compose on Android.

```fr
lottie: { src: "https://example.com/animation.json" }

lottie: {
    src: "assets/animations/loading.json"
    on_complete: animationDone()
    styles: { width: 200dp  height: 200dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Lottie JSON URL |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_complete` | No |

#### `web_view`

Embedded web browser component.

```fr
web_view: { url: "https://example.com" }

web_view: {
    url: UserStore.web_url
    styles: { width: 100%  height: 400dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `url` | String | **Yes** | — | URL to load |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `map_view`

Embedded native map. Uses MapKit on iOS, Google Maps on Android.

```fr
map_view: { lat: 37.7749  lng: -122.4194 }

map_view: {
    lat: UserStore.latitude
    lng: UserStore.longitude
    styles: { width: 100%  height: 300dp  border_radius: 12dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `lat` | Float | No | — | Latitude |
| `lng` | Float | No | — | Longitude |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | — | No |

#### `camera_view`

Live camera preview. Uses AVCaptureSession on iOS, CameraX on Android.

```fr
camera_view: {
    styles: { width: 100%  height: 300dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| All layout styles | — |

#### `qr_scanner`

QR/barcode scanner with live preview.

```fr
qr_scanner: {
    styles: { width: 100%  height: 300dp }
    on_scan: handleQRCode()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| All layout styles | `on_scan` |

---

### Gesture Components

#### `swipeable`

Wraps content with swipe gesture detection.

```fr
swipeable: {
    on_swipe: handleSwipe()
    children: [
        card: {
            children: [ text: { content: "Swipe me!" } ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_swipe` | Yes (any) |

#### `draggable`

Makes children draggable with pan gesture.

```fr
draggable: {
    on_drag: handleDrag()
    children: [
        text: { content: "Drag me around" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_drag` | Yes (any) |

#### `refresh`

Pull-to-refresh container.

```fr
refresh: {
    refreshing: UserStore.is_refreshing
    on_refresh: wait:UserStore.refresh()
    children: [
        list: {
            data: UserStore.items
            build: (item) => {
                text: { content: item.name }
            }
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `refreshing` | Bool | No | — | Whether refreshing is active |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_refresh` | Yes (any) |

#### `long_press`

Detects long-press gesture on wrapped content.

```fr
long_press: {
    on_long_press: handleLongPress()
    children: [
        card: {
            children: [ text: { content: "Press and hold" } ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout styles | `on_long_press` | Yes (any) |

---

### Plugin Component

The `plugin` component bridges to native plugin functionality.

```fr
plugin: { name: "analytics"  method: "trackEvent" }

plugin: {
    name: "frame-camera"
    method: capture
}

plugin: {
    name: "frame-storage"
    method: saveFile
}

plugin: {
    name: "frame-connectivity"
    method: isOnline
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `name` | String | **Yes** | — | Plugin name |
| `method` | String | **Yes** | — | Method to call |

| Styles | Events | Children |
|--------|--------|----------|
| — | — | No |

---

## Icon System

Frame's icon system supports three tiers of icon definition, giving you flexibility from quick prototyping to fully custom branded icons.

### Tier 1: Platform Icons (Built-in)

Use any Apple SF Symbol (iOS) or Material Design Icon (Android) directly by name:

```fr
// Uses SF Symbol "heart.fill" on iOS / Material Icon "Favorite" on Android
icon: { name: "heart"  styles: { color: "#FF0000"  width: 24  height: 24 } }

// Common icons
icon: { name: "magnifyingglass" }   // search
icon: { name: "gearshape" }          // settings
icon: { name: "house.fill" }        // home
icon: { name: "person.fill" }       // user/profile
icon: { name: "plus" }              // add
icon: { name: "trash.fill" }        // delete
icon: { name: "bell.fill" }         // notifications
icon: { name: "star.fill" }         // favorite
icon: { name: "xmark" }             // close
icon: { name: "chevron.left" }      // back
icon: { name: "checkmark" }         // check
```

### Tier 2: Icon Bundle Files (.frameicons)

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
    },
    {
      "name": "search",
      "sf_symbol": "magnifyingglass",
      "material": "Search",
      "category": "actions"
    }
  ]
}
```

**Bundle fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | **Yes** | Logical icon name (used as `name:` prop value) |
| `sf_symbol` | String | No | Apple SF Symbol name for iOS |
| `material` | String | No | Material Icon name for Android |
| `svg_path` | String | No | SVG path data for custom rendering |
| `category` | String | No | Icon category for organization |
| `tags` | [String] | No | Tags for filtering |

**Usage in Frame:**

```fr
// Uses the bundle definition matching "home"
icon: { name: "home"  styles: { width: 24  height: 24 } }

// The framework resolves "home" → SF Symbol "house.fill" on iOS
//                         → Material Icon "Home" on Android
//                         → generates custom PDF/XML asset if svg_path is set
```

**Commands:**

```bash
# Load icons from a bundle file
frame icon load-bundle path/to/icons.frameicons

# List all registered icons
frame icon list

# Generate platform icon assets (PDF for iOS, XML VectorDrawable for Android)
frame icon generate
frame icon generate --target ios
frame icon generate --target android
```

**Multiple bundle files:** Place multiple `.frameicons` files in `assets/icons/` — the system loads all of them and merges by name (first definition wins):

```
myApp/assets/icons/
├── default.frameicons       # 332 bundled icons (shipped with Frame)
├── custom.frameicons        # Your custom icon definitions
└── imported_icons.frameicons # Icons from `frame icon load-bundle`
```

### Tier 3: Custom SVG Icons

Add individual SVG icons that get registered in `frame-icons.json`:

```bash
frame icon add path/to/custom-icon.svg
frame icon add path/to/icon.svg --name "my_custom_icon"
```

This extracts the SVG `<path d="...">` data and registers it in your project's `frame-icons.json` manifest.

### Icon Asset Generation

At deploy time, icons with `svg_path` data are automatically converted to platform-native assets:

- **iOS**: SVG path data wrapped in a reference file at `Assets.xcassets/Resources/`
- **Android**: XML VectorDrawable files generated in `res/drawable/ic_{name}.xml`

```bash
# Manual generation (runs automatically during deploy)
frame icon generate
```

### 332 Built-in Icons

Frame ships with **332 pre-defined icons** organized into 14 categories, ready to use by name:

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

---

## Plugin System

Frame plugins extend the framework with native functionality, custom components, and platform permissions.

### Available Plugins

#### `frame_camera`

Camera capture plugin — take photos from device camera.

```fr
import { capture } "frame-camera"

// Capture a photo
fn handleCapture: async () => {
    :var photo = wait:capture("jpg", 0.9, "camera")
    if photo != null {
        UserStore.photo = photo
    }
}
```

#### `frame_storage`

Local file storage plugin — save, read, and manage files.

```fr
import { saveFile } "frame-storage"

fn saveData: async () => {
    :var result = wait:saveFile("notes.txt", "Hello, world!")
    if result != null {
        log.info("File saved")
    }
}
```

**Storage API:**

| Function | Description |
|----------|-------------|
| `wait:saveFile(filename, data)` | Write data to file |
| `wait:readFile(filename)` | Read file contents |
| `wait:deleteFile(filename)` | Delete a file |
| `wait:listFiles()` | List all files |
| `wait:fileExists(filename)` | Check if file exists |

#### `frame_connectivity`

Network connectivity monitoring plugin.

```fr
import { isOnline } "frame-connectivity"

fn checkNetwork: async () => {
    :var online = wait:isOnline("any")
    if online != true {
        navigate_modal("/offline")
    }
}

// Monitor changes
fn onNetworkChange: async (connected: bool) => {
    if connected {
        wait:syncData()
    }
}
```

**Connectivity API:**

| Function | Description |
|----------|-------------|
| `wait:isOnline(type)` | Check connectivity (`"any"`, `"wifi"`, `"cellular"`) |
| `wait:onNetworkChange()` | Subscribe to network state changes |

### Plugin Permissions

Plugins automatically inject required permissions:

```json
{
  "frame_camera": {
    "ios": ["NSCameraUsageDescription"],
    "android": ["android.permission.CAMERA"]
  },
  "frame_storage": {
    "android": ["android.permission.WRITE_EXTERNAL_STORAGE"]
  }
}
```

### Install Plugins

```bash
# Install via Frame Plugin Registry
frame plugin add frame_camera
frame plugin add frame_storage

# Install from GitHub
frame plugin add @user/repo
frame plugin add @user/repo@v1.2.3

# List installed plugins
frame plugin list

# Remove a plugin
frame plugin remove frame_camera
```

### Create a Plugin

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

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `frame start <name>` | Create new Frame project |
| `frame start <name> --arch clean` | Create with Clean Architecture |
| `frame start <name> --arch mvc` | Create with MVC architecture |
| `frame build` | Compile .fr files, check for errors |
| `frame build --watch` | Rebuild on file changes |
| `frame build --strict` | Treat warnings as errors |
| `frame deploy ios` | Deploy to iOS simulator (Xcode required) |
| `frame deploy android` | Deploy to Android (SDK required) |
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
| `frame icon load-bundle <path>` | Load icons from .frameicons bundle |
| `frame icon list` | List all registered icons |
| `frame icon generate` | Generate platform icon assets (PDF/XML) |
| `frame plugin add <name>` | Install a plugin |
| `frame plugin remove <name>` | Remove a plugin |
| `frame plugin list` | List installed plugins |
| `frame plugin create <name>` | Scaffold a new plugin |
| `frame init-examples` | Regenerate example projects |

### `frame.config.json`

```json
{
  "name": "myApp",
  "bundle_id": "com.example.myapp",
  "version": "1.0.0",
  "build_number": "1",
  "render_mode": "native",
  "min_android_sdk": 24,
  "min_ios": "16.0",
  "plugins": {
    "frame_camera": "0.1.0",
    "frame_storage": "0.1.0",
    "frame_connectivity": "0.1.0"
  }
}
```

---

## Architecture

```
.fr files  ──→  PEG Parser  ──→  AST  ──→  Resolver  ──→  Type Checker  ──→  Codegen
                    │                                                   │
                    │                                                   ├── Android (Kotlin/Compose)
               Component Registry                                       │
               (65+ components)                                         └── iOS (UIKit/Swift)
                    │                                                       │
               Plugin Registry                                              │
                    │                                                   Validation Files
               Icon Bundle System                                      (native validators)
                    │
               Stdlib Translator
```

### Pipeline

1. **Parser** — PEG grammar (`grammar.pest`) produces a typed AST from `.fr` source files
2. **Resolver** — Validates imports, resolves component references, validates routes
3. **Type Checker** — Enforces strict type system, validates props and styles against the registry
4. **Codegen** — Generates platform-native code:
   - **iOS**: Swift with UIKit views, Auto Layout constraints, UINavigationController routing
   - **Android**: Kotlin with Jetpack Compose, NavHost routing, Material Design theming

### Key Design Decisions

- **No runtime**: Frame compiles to native code — no interpreter, no WebView, no bridge overhead
- **Static typing**: All type checking happens at compile time
- **Platform-native icons**: Maps to SF Symbols (iOS) and Material Icons (Android) — no custom icon font needed
- **Bundle-based icon system**: `.frameicons` files provide cross-platform icon definitions alongside custom SVG support
- **Plugin architecture**: Native code modules with Frame DSL bindings, permission management, and asset distribution

---

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

```bash
# Run the test suite (415+ tests)
cargo test

# Build in release mode
cargo build --release

# Regenerate example projects
cargo run -- init-examples
```

## License

MIT License — see [LICENSE](LICENSE) for details.
