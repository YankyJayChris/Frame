# Store / State Management

Frame uses `:store` blocks for global state management. Each store is a named slice containing typed fields, actions (functions), and optional persistence strategies.

## Basic Store

```fr
:store UserStore {
    user_name: string = "Jane Smith"
    user_email: string = "jane@example.com"
    user_bio: string = "Full-stack developer"
    is_loading: bool = false
    error: string = ""
    dark_mode: bool = false
    volume: float = 50
}
```

## Store with Actions

Stores can contain functions (actions) that mutate state:

```fr
:store UserStore {
    user_name: string = "Jane Smith"
    user_email: string = "jane@example.com"
    user_bio: string = "Full-stack developer"
    is_loading: bool = false
    error: string = ""

    fn load: async (id: string) => {
        UserStore.is_loading = true
        UserStore.error = ""
        try {
            result = wait:fetch("/api/users/\(id)", { method: "GET" })
            if result != null {
                UserStore.user_name = result.name
                UserStore.user_email = result.email
                UserStore.user_bio = result.bio
            }
        } catch (err) {
            UserStore.error = "Failed to load: \(err)"
        }
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

    fn reset: () => {
        UserStore.user_name = ""
        UserStore.user_email = ""
        UserStore.user_bio = ""
        UserStore.is_loading = false
        UserStore.error = ""
    }
}
```

## Store with Persistence

Use the `persist:` block to persist field values across app launches:

```fr
:store AuthStore {
    token: string = ""
    refresh_token: string = ""
    theme: string = "light"
    last_user_id: string = ""
    notifications_enabled: bool = true

    persist: {
        token: secure             // Keychain (iOS) / EncryptedSharedPreferences (Android)
        refresh_token: secure
        theme: local              // UserDefaults (iOS) / SharedPreferences (Android)
        last_user_id: local
        notifications_enabled: local
    }
}
```

| Strategy | Android                          | iOS                     |
|----------|----------------------------------|-------------------------|
| `local`  | SharedPreferences                | UserDefaults            |
| `secure` | EncryptedSharedPreferences       | Keychain Services       |

## Reading Store State in UI

Access store fields directly using `StoreName.field_name`:

```fr
page: {
    name: "Profile"
    route: "/profile"
    children: [
        column: {
            styles: { padding: 16dp  gap: 12dp }
            children: [
                // Loading indicator
                progress_circle: { show_if: UserStore.is_loading }

                // User info
                text: {
                    content: UserStore.user_name
                    styles: { font_size: 18sp  font_weight: "bold" }
                }
                text: {
                    content: UserStore.user_email
                    styles: { font_size: 14sp  color: "#666" }
                }

                // Error display
                text: {
                    content: UserStore.error
                    styles: { color: "#FF0000" }
                    show_if: UserStore.error != ""
                }

                // Interactive controls bound to store
                switch: {
                    value: UserStore.dark_mode
                    on_change: wait:UserStore.toggleDarkMode()
                }
                slider: {
                    value: UserStore.volume
                    min: 0
                    max: 100
                    on_change: wait:UserStore.setVolume(value: UserStore.volume)
                }
            ]
        }
    ]
}
```

## Calling Store Actions

Store actions are called with the `wait:` prefix if async, or directly if sync:

```fr
// Sync action
button: { content: "Reset"  on_click: UserStore.reset() }

// Async action — must use wait:
button: { content: "Load"  on_click: wait:UserStore.load(id: "42") }

// Toggle with wait:
switch: { value: UserStore.dark_mode  on_change: wait:UserStore.toggleDarkMode() }
```

## Store Field Types

| Type     | Description      |
|----------|------------------|
| `string` | Text value       |
| `int`    | 64-bit integer   |
| `float`  | 64-bit float     |
| `bool`   | Boolean          |
| `object` | Key-value map    |
| `list`   | Ordered list     |

## Store Syntax Reference

```fr
:store StoreName {
    field_name: type = default_value

    persist: {
        field_name: local     // or secure
    }

    fn actionName: async (param: type) => {
        // Mutate fields using StoreName.field = expr
    }
}
```

## Best Practices

- Keep stores focused on a single domain (user, settings, etc.)
- Use `persist: { ... }` sparingly — only for values that must survive app restarts
- Access store fields directly in UI — reactivity is automatic
- Call async actions with `wait:` prefix
- Initialize fields with sensible defaults
