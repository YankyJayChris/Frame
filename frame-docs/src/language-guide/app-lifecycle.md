# App Lifecycle

The `:app` block declares app-level lifecycle hooks. It is defined once, typically in `src/project.fr`.

## Basic App Lifecycle

```fr
:app {
    on_launch:     appInit
    on_foreground: appForeground
    on_background: appBackground
}

fn appInit: () => {
    log.info("App launched")
    // Initialize SDKs, load settings, check auth
    UserStore.loadPreferences()
}

fn appForeground: () => {
    log.info("App foregrounded")
    // Refresh data, resume timers
    AuthStore.refreshToken()
}

fn appBackground: () => {
    log.info("App backgrounded")
    // Save state, pause operations
    UserStore.savePreferences()
}
```

## Full Example

```fr
:vars {
    $primary: "#3584e4"
    $surface: "#ffffff"
}

:app {
    on_launch:     initApp
    on_foreground: handleForeground
    on_background: handleBackground
}

import { text, button, column, scaffold, app_bar } "frame-core"
import { UserStore } "@/stores/user_store"
import { AuthStore } "@/stores/auth_store"

fn initApp: () => {
    log.info("Application starting")
    UserStore.loadPreferences()
    AuthStore.checkAuth()
    registerPushNotifications()
}

fn handleForeground: () => {
    log.info("App returned to foreground")
    AuthStore.refreshToken()
    UserStore.refreshData()
}

fn handleBackground: () => {
    log.info("App moving to background")
    UserStore.savePreferences()
    UserStore.saveScrollPosition()
}

fn registerPushNotifications: () => {
    log.info("Notifications registered")
}

page: {
    name: "Home"
    route: "/"
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

## App Lifecycle Reference

| Hook | Timing | Android | iOS |
|------|--------|---------|-----|
| `on_launch` | App starts | `Application.onCreate()` | `didFinishLaunchingWithOptions` |
| `on_foreground` | App becomes visible | `ProcessLifecycleOwner ON_START` | `sceneWillEnterForeground` |
| `on_background` | App moves to background | `ProcessLifecycleOwner ON_STOP` | `sceneDidEnterBackground` |

## Hook Functions

The lifecycle hooks reference functions defined elsewhere in the project. These functions can be:

- Defined in `project.fr` (top-level)
- Imported from other files
- Sync or async

```fr
:app {
    on_launch:     initApp
    on_foreground: wait:refreshData
    on_background: saveState
}

fn initApp: () => {
    log.info("App initialized")
}

fn refreshData: async () => {
    wait:UserStore.refreshAll()
}

fn saveState: () => {
    UserStore.persistState()
}
```

## Best Practices

- Keep `on_launch` lightweight — initialize SDKs and check auth status
- Use `on_foreground` to refresh stale data
- Use `on_background` to save critical state
- Avoid heavy computation in lifecycle hooks
- The `:app` block must appear exactly once per project
