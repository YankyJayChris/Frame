# Navigation

Frame provides navigation functions for moving between pages, presenting modals, and managing the navigation stack.

## Basic Navigation

```fr
// Navigate to a route
navigate("/profile")
navigate("/home")
navigate("/settings")

// With path parameter
navigate("/profile/\(userId)")
navigate("/products/\(productId)")
```

## Navigation with Options

```fr
// Clear the entire back stack
navigate("/home", clear_stack: true)

// Replace the current screen
navigate("/login", replace: true)

// Avoid duplicate screens (single top)
navigate("/detail", single_top: true)

// Animated transition
navigate("/detail", transition: "slide_up")
navigate("/settings", transition: "fade")
navigate("/profile", transition: "slide_left")
```

## Navigate Replace

Replace the current screen in the navigation stack (no back button to previous screen):

```fr
navigate_replace("/login")
navigate_replace("/onboarding/complete")
```

## Navigate Back

```fr
// Go back one screen
navigate_back()

// Used in event handlers
button: { content: "Back"  on_click: navigate_back() }
```

## Navigate Back To

Pop the navigation stack to a specific route:

```fr
navigate_back_to("/home")
navigate_back_to("/dashboard")
```

## Navigate Modal

Present a page as a modal sheet:

```fr
navigate_modal("/settings")
navigate_modal("/create-post")
navigate_modal("/filter")
```

## Navigate Dismiss

Dismiss the currently presented modal:

```fr
navigate_dismiss()

button: { content: "Close"  on_click: navigate_dismiss() }
```

## Navigation in Event Handlers

```fr
page: {
    name: "Home"
    route: "/"
    children: [
        column: {
            styles: { padding: 16dp  gap: 12dp }
            children: [
                button: {
                    content: "View Profile"
                    on_click: navigate("/profile/\(userId)")
                }
                button: {
                    content: "Settings"
                    on_click: navigate_modal("/settings")
                }
                button: {
                    content: "Logout"
                    on_click: navigate_replace("/login")
                }
                icon: {
                    name: "arrow.left"
                    on_click: navigate_back()
                }
            ]
        }
    ]
}
```

## Navigation in Functions

```fr
fn handleLogin: async () => {
    :var result = wait:fetch("/api/auth/login", {
        method: "POST"
        headers: {
            Content-Type: "application/json"
        }
        body: {
            email: UserStore.email
            password: UserStore.password
        }
    })
    if result != null {
        navigate("/home", clear_stack: true)
    } else {
        UserStore.error = "Login failed"
    }
}

fn goToProfile: (userId: string) => {
    navigate("/profile/\(userId)")
}

fn closeAndGoHome: () => {
    navigate_dismiss()
    navigate("/home")
}
```

## Navigation Reference

| Function | Description | Signature |
|----------|-------------|-----------|
| `navigate` | Push a route onto the stack | `navigate(route, opts?)` |
| `navigate_replace` | Replace current screen | `navigate_replace(route)` |
| `navigate_back` | Pop one screen | `navigate_back()` |
| `navigate_back_to` | Pop to a specific route | `navigate_back_to(route)` |
| `navigate_modal` | Present modally | `navigate_modal(route)` |
| `navigate_dismiss` | Dismiss modal | `navigate_dismiss()` |

### Navigate Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `clear_stack` | bool | `false` | Clear entire back stack |
| `replace` | bool | `false` | Replace current entry |
| `single_top` | bool | `false` | Avoid duplicate screens |
| `transition` | string | `nil` | `"slide_up"`, `"fade"`, `"slide_left"` |

## Platform Mapping

| Function | Android | iOS |
|----------|---------|-----|
| `navigate` | `navController.navigate(route)` | `pushViewController(routeVC)` |
| `navigate_replace` | `popUpTo(current) { inclusive=true }` | Pop + push |
| `navigate_back` | `navController.popBackStack()` | `popViewController(animated:)` |
| `navigate_back_to` | `popBackStack(route, false)` | `popToViewController` |
| `navigate_modal` | `navController.navigate(route)` | `present(routeVC)` |
| `navigate_dismiss` | `popBackStack()` | `dismiss(animated:)` |
