# blog-app

A Frame cross-platform mobile app using **MVC**.

---

## Project Structure

```
src/
  models/           # :obj types + :store state
  views/
    pages/          # HomePage.fr (Home + Profile pages)
    components/     # UserCard.fr
  controllers/      # UserController.fr
  tests/            # UserStore, api, navigation tests
```

## Features Demonstrated

- **`:obj`** — typed data models → Kotlin `data class` / Swift `struct`
- **`:store`** — reactive state with typed fields and async actions
- **`:vars`** / **`:breakpoints`** — design tokens and responsive breakpoints
- **`:var`** — typed local variables (immutable by default, `:var mut` for reassignment)
- **`:app {}`** — app-level lifecycle hooks (`on_launch`, `on_foreground`, `on_background`)
- **`import`** — cross-file imports for components, stores, functions, and plugins
- **`show_if`** — conditional rendering based on store state
- **`try`/`catch`/`finally`** — error handling in async operations
- **`wait:fetch`** — HTTP API calls with mock support in tests
- **Typed route params** — `page { params: { userId: string } }` → typed Screen / ViewController
- **Navigation options** — `navigate("/path", replace: true)`, `clear_stack`, `single_top`, `transition`
- **navigate_back_to** — pop to any named route in the back stack
- **navigate_modal / navigate_dismiss** — modal presentation and dismissal
- **Component lifecycle** — `on_mount`, `on_update`+`watch`, `on_unmount` on any node
- **Page lifecycle** — `before_enter`, `on_mount`, `on_unmount`, `on_foreground`, `on_background`
- **Plugin params** — all params caller-supplied, validated at runtime, no hardcoding
- **Strict typing** — every variable, store field, function param, and component prop is type-checked

## App Lifecycle

```fr
// project.fr — declared once, wired into Application / AppDelegate
:app {
    on_launch:     appInit      // Application.onCreate / didFinishLaunching
    on_foreground: appForeground // ProcessLifecycleOwner ON_START / sceneWillEnterForeground
    on_background: appBackground // ProcessLifecycleOwner ON_STOP / sceneDidEnterBackground
}
```

## Navigation

### Page with typed route params
```fr
page: {
    name: "Profile"
    route: "/profile/:userId"
    params: { userId: string }      // generates typed Screen/ViewController params
    before_enter: checkAuth           // guard — any expression, not just string names
    on_mount:     loadProfile         // viewDidAppear / LaunchedEffect "mount"
    before_leave: saveEdits           // viewDidDisappear / DisposableEffect
}
```

### Navigation options
```fr
// Push (default)
navigate("/dashboard")

// Replace current entry — back won't return here
navigate("/home", replace: true)

// Clear entire stack before navigating (login → main flow)
navigate("/home", clear_stack: true)

// Prevent duplicate screens
navigate("/search", single_top: true)

// Custom transition animation
navigate("/detail", transition: "slide_up")

// Pop one entry
navigate_back()

// Pop to a specific route
navigate_back_to("/home")

// Present modally (sheet / dialog)
navigate_modal("/settings")
navigate_dismiss()
```

### Component lifecycle
```fr
column: {
    on_mount:   startPolling      // LaunchedEffect(Unit) / DispatchQueue.main.async
    on_update:  refreshData       // LaunchedEffect(key) fires when watch dependency changes
    watch:      UserStore.items   // dependency key for on_update
    on_unmount: stopPolling       // DisposableEffect/onDispose on Android
    children: [...]
}
```

## Type System

| Type | Description | Kotlin | Swift |
|------|-------------|--------|-------|
| `string` | UTF-8 text | `String` | `String` |
| `int` | Integer | `Int` | `Int` |
| `float` | Floating-point | `Float` | `Double` |
| `bool` | Boolean | `Boolean` | `Bool` |
| `object` | Key-value map | `Any` | `[String: Any]?` |
| `list` | Ordered array | `List<Any>` | `[Any]?` |
| `nullable(T)` | Nullable variant | `T?` | `T?` |

## Plugins

### `frame_camera`
Captures photos — format, quality, and source are **caller-supplied and validated**.
```fr
import { capture } "frame-camera"
:var photo = wait:capture("jpg", 0.9, "camera")  // format / quality / source
```
- `format`: `"jpg"` | `"png"` | `"webp"` (default `"jpg"`)
- `quality`: `0.0`–`1.0` (default `0.8`)
- `source`: `"camera"` | `"gallery"` (default `"camera"`)

### `frame_storage`
Saves, loads, and deletes files — directory and encoding are **caller-supplied and validated**.
```fr
import { saveFile, loadFile, deleteFile } "frame-storage"
wait:saveFile("notes.txt", "hello", "documents", "utf8")
:var content = wait:loadFile("notes.txt", "documents", "utf8")
```
- `directory`: `"documents"` | `"cache"` | `"temp"` (default `"documents"`)
- `encoding`: `"utf8"` | `"base64"` (default `"utf8"`)
- Filenames validated — empty names and path separators rejected.

### `frame_connectivity`
Monitors network state — type filter and poll interval are **caller-supplied and validated**.
```fr
import { isOnline, onNetworkChange } "frame-connectivity"
:var online = wait:isOnline("wifi")            // type: any | wifi | cellular
wait:onNetworkChange("any", 10)                // interval: 1–60 s
```

Plugin source files are auto-copied during `frame deploy`.

## Error Handling

```fr
fn loadUser: async (id: string) => {
    try {
        UserStore.user = wait:fetch("/api/users/$id")
    } catch (err) {
        UserStore.error = err
    } finally {
        UserStore.is_loading = false
    }
}
```

## Async / Await

```fr
fn fetchData: async (url: string) => {
    :var result = wait:fetch(url, { method: "GET" })
    return result
}
```
Async functions must be called with `wait:` prefix. Calling without `wait:` is a **compile error**.

## Commands

```bash
frame check                 # verify build environment
frame build                 # compile .fr files
frame test                  # run test suites (UserStore, api, navigation)
frame deploy android        # generate + build Android project
frame deploy ios            # generate + build iOS project
frame preview               # hot-reload dev server
frame plugin create <name>  # create a new plugin
frame plugin add <name>     # install a plugin
frame plugin add @user/repo # install from GitHub
frame plugin list           # list installed plugins
```
