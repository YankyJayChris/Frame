# Frame Framework Documentation

Last Updated: March 27, 2025

## Introduction
The Frame framework is a lightweight, declarative framework for building cross-platform applications using a custom .fr syntax. It supports desktop, Android, and iOS with a single codebase, leveraging Rust for performance and native plugins for platform-specific features. This document details all features, syntax, and provides a complete blog app example.

## Features Overview
Frame includes 23 core features:
- Variables: Global constants with substitution.
- Internationalization (i18n): Multi-language support.
- Imports: Modular file inclusion.
- Pages: Route-based UI definitions.
- Components: Reusable UI blocks (custom and native).
- Props: Parameter passing to components.
- Styling: CSS-like styles with variables.
- Content: Text with i18n support.
- Children: Nested component hierarchies.
- Animations: Transition effects.
- Events: User and lifecycle handlers.
- Functions: Custom logic and expressions.
- State Management: Implicit state via functions.
- Navigation: Route switching.
- Fetch: HTTP requests.
- Forms: Input handling with validation.
- Lists: Dynamic rendering.
- Testing: Jest-like test suites.
- Hot Reload: Real-time preview (CLI).
- Deployment: Multi-platform builds (CLI).
- Camera Plugin: Access device camera.
- Location Plugin: Retrieve device location.
- Notification Plugin: Send notifications.
- Touch Events: Built-in touch gesture handling (start, move, end, cancel).
- Built-in Components: Predefined components like `AppBar`, `Text`, `Button`, etc.
- Built-in Plugins: Camera, Location, Notification.

## Syntax and Features

### 1. Variables
Define global variables with `:vars` block.

**Syntax:**
```
:vars {
  $name: "value";
}
```

**Example:**
```
:vars {
  $primary: "#007BFF";
  $spacing: "10dp";
}
```

**Usage:** Variables are referenced with `$` prefix (e.g., `$primary`).

### 2. Internationalization (i18n)
Add translations with `:i18n` block.

**Syntax:**
```
:i18n {
  key: "value";
}
```

**Example:**
```
:i18n {
  home: "Home";
  submit: "Submit";
}
```

**Usage:** Use `t:"key"` in content to reference translations.

### 3. Imports
Include external `.fr` files or specific components.

**Syntax:**
```
import "path"
import { ident as alias } "path"
```

**Example:**
```
import "components.fr"
import { Card as BlogCard } "./components/Card.fr"
```

### 4. Pages
Define routes and UI with `page:` block.

**Syntax:**
```
page: {
  name: "page_name"
  route: "/path"
  styles: { key: "value" }
  children: [
    Component: { ... }
  ]
}
```

**Example:**
```
page: {
  name: "Home"
  route: "/"
  styles: { background: "#F5F5F5" }
  children: [
    Text: { content: "Welcome" }
  ]
}
```

### 5. Components
Create reusable UI elements (custom or native).

**Syntax:**
```
component Name: {
  props: { key: type = "default" }
  styles: { key: "value" }
  content: "text"
  children: [ ... ]
}
```

**Example:**
```
component Card: {
  props: { title: string = "Untitled" }
  styles: { width: "200dp" }
  content: "$title"
}
```

**Native Example:**
```
Text: { content: "Hello" }
```

### 6. Props
Pass parameters to components.

**Syntax:**
```
props: { name: type = "default" }
```

**Example:**
```
component Button: {
  props: { label: string = "Click Me" }
  content: "$label"
}
```

### 7. Styling
Apply CSS-like styles.

**Syntax:**
```
styles: { property: "value" }
```

**Example:**
```
Text: {
  styles: { color: "$primary"; margin: "10dp" }
  content: "Styled"
}
```

**Units:** dp, px, %, ms.

### 8. Content
Set text content with i18n support.

**Syntax:**
```
content: "text" | t:"key" | "$variable"
```

**Example:**
```
Text: { content: t:"home" }
```

### 9. Children
Nest components.

**Syntax:**
```
children: [
  Component: { ... }
]
```

**Example:**
```
Container: {
  children: [
    Text: { content: "Child 1" },
    Text: { content: "Child 2" }
  ]
}
```

### 10. Animations
Add transition effects.

**Syntax:**
```
animate: {
  from: { key: "value" }
  to: { key: "value" }
  duration: number unit
  easing: "type"
}
```

**Example:**
```
Button: {
  content: "Fade"
  animate: {
    from: { opacity: "0" }
    to: { opacity: "1" }
    duration: 500ms
    easing: "ease-in"
  }
}
```

### 11. Events
Handle interactions and lifecycle.

**Syntax:**
```
on_click: "function:()"
on_change: "function:($value)"
```

**Example:**
```
Button: {
  content: "Click"
  on_click: "handle_click:()"
}
```

### 12. Functions
Define custom logic.

**Syntax:**
```
fn name:(param:type) => {
  expression
}
```

**Example:**
```
fn update:(value:string) => {
  if: value === "test" {
    return result = "matched"
  }
}
```

**Expressions:** return, call, if, for, fetch, etc.

### 13. State Management
Implicit via functions and `self`.

**Example:**
```
fn set_text:(text:string) => {
  return self.text = text
}
```

### 14. Navigation
Switch routes.

**Example:**
```
fn go_home:() => {
  navigate("/home")
}
```

### 15. Fetch
Make HTTP requests.

**Syntax:**
```
fetch:("url", "options")
```

**Example:**
```
fn load_data:() => {
  fetch:("https://api.example.com", "method:GET")
}
```

### 16. Forms
Handle inputs with validation.

**Syntax:**
```
form: {
  validation: "rule"
  on_submit: "function:()"
  children: [ ... ]
}
```

**Example:**
```
form: {
  validation: "required"
  on_submit: "submit:()"
  children: [
    input: { value: "Name" }
  ]
}
```

### 17. Lists
Render dynamic lists.

**Syntax:**
```
list: {
  data: source
  build:(item) => {
    Component: { ... }
  }
}
```

**Example:**
```
list: {
  data: posts
  build:(post) => {
    Card: { content: "$post.title" }
  }
}
```

### 18. Testing
Write tests.

**Syntax:**
```
describe: "name" => {
  it: "case" => {
    expect: target.method:("value")
  }
}
```

**Example:**
```
describe: "Tests" => {
  it: "click works" => {
    expect: button.on_click:("handle_click")
  }
}
```

### 19. Hot Reload
CLI command: `frame preview`

**Description:** Real-time updates during development.

### 20. Deployment
CLI command: `frame deploy [target]`

**Targets:** desktop, android, ios

### 21. Camera Plugin
Access device camera.

**Syntax:** `camera:()`

**Example:**
```
fn open_camera:() => {
  return result = camera:()
}
Button: {
  content: "Open Camera"
  on_click: "open_camera:()"
}
```

**Platforms:** Android (JNI), iOS (Objective-C), Desktop (mock).

### 22. Location Plugin
Get device location.

**Syntax:** `location:()`

**Example:**
```
fn get_location:() => {
  return loc = location:()
}
Button: {
  content: "Get Location"
  on_click: "get_location:()"
}
```

**Platforms:** Android (JNI), iOS (Objective-C), Desktop (mock).

### 23. Notification Plugin
Send notifications.

**Syntax:** `notification:("message")`

**Example:**
```
fn send_notification:(msg:string) => {
  return result = notification:(msg)
}
Button: {
  content: "Notify"
  on_click: "send_notification:(\"Hello\")"
}
```

**Platforms:** Android (JNI), iOS (Objective-C), Desktop (console).

### 24. Touch Events
Handle touch gestures directly on components, similar to `on_click`. These events are built-in and provide coordinates (`$touch_x`, `$touch_y`) to the handler functions.

**Supported Events:**
- `on_touch_start`: Fired when a touch begins, provides `$touch_x` and `$touch_y`.
- `on_touch_move`: Fired when a touch moves, provides `$touch_x` and `$touch_y`.
- `on_touch_end`: Fired when a touch ends, provides `$touch_x` and `$touch_y`.
- `on_touch_cancel`: Fired when a touch is canceled, provides `$touch_x` and `$touch_y`.

**Syntax:**
```
Component: {
  content: "Interact"
  on_touch_start: "function:($touch_x, $touch_y)"
}
```

**Example:**
```
fn handle_swipe:(x:number, y:number) => {
  return log = "Swiped to ($x, $y)"
}

Text: {
  content: "Swipe Here"
  on_touch_move: "handle_swipe:($touch_x, $touch_y)"
  styles: { x: "10dp"; y: "50dp" }
}
```

**Platforms:** Integrated into the runtime with support for Android, iOS, and Desktop (coordinates passed via the `CanvasApp` `on_touch` method).

### 25. Built-in Components
Frame provides a set of predefined components to accelerate development. These components support props, styles, children (where applicable), and events like `on_click` and touch handlers.

**Components:**
- `AppBar`: A top bar with title, menu, and actions.
- `Text`: Displays text (no children).
- `Image`: Renders images or SVGs (no children).
- `Button`: A clickable button with text or icon (no children).
- `List`: Renders a dynamic list from data with a build function.
- `BottomBar`: A navigation bar with selectable items.
- `Input`: A text input field (no children).
- `Dropdown`: A selectable dropdown menu (no children).
- `Form`: A container for form elements with validation and submission.

**Syntax:**
```
ComponentName: {
  prop1: "value"
  styles: { key: "value" }
  on_click: "function:()"
  children: [
    AnotherComponent: { ... }
  ]
}
```

**Examples:**

**AppBar:**
```
AppBar: {
  title: "My App"
  styles: { x: "0dp"; y: "0dp"; background: "$primary" }
  on_touch_move: "handle_swipe:($touch_x, $touch_y)"
  children: [
    Text: { content: "Subtitle" }
  ]
}
```

**Text:**
```
Text: {
  content: "Hello World"
  styles: { x: "10dp"; y: "60dp"; color: "#FFF" }
  on_click: "log_click:()"
}
```

**Image:**
```
Image: {
  src: "icon.svg"
  styles: { x: "10dp"; y: "100dp"; width: "50dp"; height: "50dp" }
  on_touch_end: "log_touch:($touch_x, $touch_y)"
}
```

**Button:**
```
Button: {
  content: "Click Me"
  icon: "click.svg"
  styles: { x: "10dp"; y: "160dp"; background: "$primary" }
  on_click: "button_click:()"
}
```

**List:**
```
List: {
  data: ["Item 1", "Item 2", "Item 3"]
  build: "item => Text: { content: item }"
  styles: { x: "10dp"; y: "220dp" }
  children: [
    Text: { content: "List Footer" }
  ]
}
```

**BottomBar:**
```
BottomBar: {
  current: "0"
  items: [
    { content: "Home"; icon: "home.svg"; on_click: "navigate:(\"/\")" },
    { content: "Profile"; icon: "profile.svg"; on_click: "navigate:(\"/profile\")" }
  ]
  styles: { y: "550dp"; background: "$primary" }
  on_touch_start: "log_touch:($touch_x, $touch_y)"
}
```

**Input:**
```
Input: {
  value: "Enter text"
  styles: { x: "10dp"; y: "300dp"; width: "200dp" }
  on_change: "update_input:($value)"
}
```

**Dropdown:**
```
Dropdown: {
  options: ["Option 1", "Option 2", "Option 3"]
  selected: "1"
  styles: { x: "10dp"; y: "350dp" }
  on_select: "select_option:($index)"
}
```

**Form:**
```
Form: {
  validation: "required"
  on_submit: "submit_form:()"
  styles: { x: "10dp"; y: "400dp" }
  children: [
    Input: { value: "Name"; on_change: "update_name:($value)" },
    Button: { content: "Submit"; on_click: "submit_form:()" }
  ]
}
```

**Platforms:** These components are fully supported across Android, iOS, and Desktop, with rendering handled by the `Canvas` API.

## Blog App Example
Below is a complete blog app using Frame features, including built-in components.

```
:vars {
  $primary: "#007BFF";
  $spacing: "10dp";
}

:i18n {
  home: "Home";
  posts: "Posts";
  profile: "Profile";
  about: "About";
  submit: "Submit";
  camera: "Open Camera";
  location: "Get Location";
  notify: "Send Notification";
}

page: {
  name: "Home"
  route: "/"
  styles: { direction: "column" }
  children: [
    AppBar: {
      title: t:"home"
      styles: { background: "$primary" }
      on_touch_move: "handle_swipe:($touch_x, $touch_y)"
    },
    Text: {
      content: "Welcome to the Blog"
      styles: { x: "$spacing"; y: "60dp"; font_size: "20dp" }
    },
    Button: {
      content: t:"posts"
      styles: { x: "$spacing"; y: "100dp"; background: "$primary"; color: "#FFF" }
      on_click: "navigate:(\"/posts\")"
    }
  ]
}

page: {
  name: "Posts"
  route: "/posts"
  styles: { direction: "column" }
  before_enter: "load_posts:()"
  children: [
    AppBar: { title: t:"posts"; styles: { background: "$primary" } },
    List: {
      data: "$posts"
      build: "post => Text: { content: post }"
      styles: { x: "$spacing"; y: "60dp" }
    }
  ]
}

page: {
  name: "Profile"
  route: "/profile"
  styles: { direction: "column" }
  children: [
    AppBar: { title: t:"profile"; styles: { background: "$primary" } },
    Form: {
      validation: "required"
      on_submit: "submit_profile:()"
      styles: { x: "$spacing"; y: "60dp" }
      children: [
        Input: {
          value: "Enter name"
          on_change: "update_name:($value)"
          styles: { width: "200dp" }
        },
        Button: {
          content: t:"submit"
          styles: { y: "40dp"; background: "$primary"; color: "#FFF" }
        },
        Button: {
          content: t:"location"
          on_click: "get_location:()"
          styles: { y: "80dp"; background: "$primary"; color: "#FFF" }
        },
        Text: {
          content: "Swipe to Test Touch"
          on_touch_move: "handle_swipe:($touch_x, $touch_y)"
          styles: { y: "120dp"; font_size: "16dp" }
        }
      ]
    }
  ]
}

page: {
  name: "About"
  route: "/about"
  styles: { direction: "column" }
  children: [
    AppBar: { title: t:"about"; styles: { background: "$primary" } },
    Text: {
      content: "About this blog"
      styles: { x: "$spacing"; y: "60dp" }
    },
    Image: {
      src: "logo.svg"
      styles: { x: "$spacing"; y: "100dp"; width: "100dp"; height: "100dp" }
    }
  ]
}

BottomBar: {
  current: "0"
  items: [
    { content: t:"home"; icon: "home.svg"; on_click: "navigate:(\"/\")" },
    { content: t:"posts"; icon: "posts.svg"; on_click: "navigate:(\"/posts\")" },
    { content: t:"profile"; icon: "profile.svg"; on_click: "navigate:(\"/profile\")" },
    { content: t:"about"; icon: "about.svg"; on_click: "navigate:(\"/about\")" }
  ]
  styles: { y: "550dp"; background: "$primary" }
}

fn navigate:(path:string) => {
  navigate(path)
}

fn update_name:(value:string) => {
  state.name = value
}

fn submit_profile:() => {
  log = "Profile submitted: $state.name"
}

fn load_posts:() => {
  fetch "https://api.example.com/posts" {
    then: "posts => state.posts = posts"
  }
}

fn open_camera:() => {
  camera:capture()
}

fn get_location:() => {
  location:get()
}

fn send_notification:(msg:string) => {
  notification:send(msg)
}

fn handle_swipe:(x:number, y:number) => {
  return log = "Swiped to ($x, $y)"
}

fn select_option:(index:number) => {
  log = "Selected option at index $index"
}

describe: "Blog Tests" => {
  it "navigates to posts" => {
    assert navigate("/posts") == "/posts"
  }
}
```

## Getting Started
1. **Install Rust and Frame:**
   - `cargo new my-app`
   - `cd my-app`
   - Add to `Cargo.toml`: `frame = { path = "../frame" }`
2. **Create `src/project.fr`** with the above blog app.
3. **Build and Run:**
   - `frame build`
   - `cargo run`
4. **Preview Changes:**
   - `frame preview`
5. **Deploy:**
   - `frame deploy desktop`
   - `frame deploy android`
   - `frame deploy ios`

## Supported Platforms
- **Desktop:** Windows, macOS, Linux
- **Android:** Via JNI plugins
- **iOS:** Via Objective-C plugins

## Contributing
Frame is developed by IGIHOZO Jean Christian. Contributions welcome via pull requests to the frame repository.

## License
MIT License