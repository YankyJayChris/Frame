# Pages

Pages are the routed entry points of a Frame application. Each page has a `name`, `route`, optional lifecycle hooks, typed route parameters, and a children tree.

## Basic Page

```fr
page: {
    name: "Home"
    route: "/"
    styles: { safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: { title: "Home" }
                column: {
                    styles: { padding: 16dp }
                    children: [
                        text: { content: "Welcome to Frame!" }
                    ]
                }
            ]
        }
    ]
}
```

## Page with Route Parameters

```fr
page: {
    name: "Profile"
    route: "/profile/:userId"
    params: { userId: string }
    styles: { safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: { title: "Profile"  leading: "chevron.left" }
                column: {
                    styles: { padding: 16dp }
                    children: [
                        text: { content: "User ID: \(userId)" }
                    ]
                }
            ]
        }
    ]
}
```

## Full Page with Lifecycle

```fr
page: {
    name: "Dashboard"
    route: "/dashboard"
    params: { section: string? }

    before_enter: checkAuth()
    before_leave: saveState()
    on_mount: loadDashboardData()
    on_background: pausePolling()
    on_foreground: resumePolling()
    on_unmount: cleanup()

    styles: {
        safe_area: true
        background: "#F5F5F5"
    }

    state: {
        items: list = []
        isLoading: bool = false
    }

    children: [
        scaffold: {
            children: [
                app_bar: { title: "Dashboard" }
                list: {
                    data: state.items
                    build: (item) => {
                        text: { content: item.name }
                    }
                }
            ]
        }
    ]
}
```

## Page Properties Reference

### `name`

Display name for the page — used internally for logging and debugging.

```fr
name: "Splash"
```

### `route`

The URL route pattern. Supports dynamic segments with `:` prefix.

```fr
route: "/"
route: "/profile/:userId"
route: "/products/:categoryId/:productId"
```

### `params`

Typed route parameters matching the dynamic segments in `route`.

```fr
params: { userId: string }
params: { categoryId: string  productId: string }
params: { section: string? }  // optional param
```

### `before_enter`

Guard function called before the page transition completes. Return `false` or call `navigate()` to redirect.

```fr
before_enter: checkAuth()
before_enter: redirectIfLoggedIn()
```

### `before_leave`

Called when the user navigates away from this page. Used for cleanup or save operations.

```fr
before_leave: saveFormDraft()
before_leave: confirmExit()
```

### `on_mount`

Called once after the page is fully visible and rendered.

```fr
on_mount: loadInitialData()
on_mount: trackPageView()
```

### `on_background`

Called when the app goes to the background while this page is visible.

```fr
on_background: pauseVideo()
on_background: saveScrollPosition()
```

### `on_foreground`

Called when the app returns to the foreground while this page is visible.

```fr
on_foreground: refreshData()
on_foreground: resumeVideo()
```

### `on_unmount`

Called just before the page is fully removed from the navigation stack.

```fr
on_unmount: cleanupSubscriptions()
on_unmount: saveAnalytics()
```

### `styles`

Style block applied to the root page container.

```fr
styles: {
    safe_area: true
    background: "#FFFFFF"
    padding: 16dp
}
```

### `state`

Local page state with typed fields and default values.

```fr
state: {
    items: list = []
    selectedId: string = ""
    isLoading: bool = false
    error: string? = null
}
```

Access state fields using `state.` prefix within the page:

```fr
text: { content: state.selectedId  show_if: state.selectedId != "" }
button: { content: "Load"  on_click: loadData() }
```

### `children`

The UI tree rendered on this page.

```fr
children: [
    scaffold: {
        children: [
            app_bar: { title: "My App" }
            column: {
                styles: { padding: 16dp  gap: 12dp }
                children: [
                    text: { content: "Hello" }
                    button: { content: "Go"  on_click: navigate("/next") }
                ]
            }
        ]
    }
]
```
