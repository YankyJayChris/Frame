# Conditionals and Loops

Frame provides `show_if` for conditional rendering, `if/else` for control flow, `for` loops for iteration, and `switch/case` for pattern matching.

## Conditional Rendering with `show_if`

The `show_if` prop conditionally renders any component:

```fr
text: { content: "Loading..."  show_if: UserStore.is_loading }

skeleton: { show_if: UserStore.is_loading }

UserCard: {
    name: UserStore.user_name
    show_if: UserStore.user_name != ""
}

text: {
    content: UserStore.error
    styles: { color: "#FF0000" }
    show_if: UserStore.error != ""
}

button: {
    content: "Submit"
    show_if: isFormValid
}
```

## if/else Statements

Use `if/else` inside function bodies for control flow:

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

### if/else with Multiple Branches

```fr
fn getDiscount: (total: float) => {
    if total > 1000 {
        return 0.2
    } else if total > 500 {
        return 0.1
    } else if total > 100 {
        return 0.05
    } else {
        return 0.0
    }
}
```

### if/else with Assignments

```fr
fn processOrder: (items: list) => {
    :var mut total = 0
    for item in items {
        total = total + item.price
    }

    :var shipping: float
    if total > 50 {
        shipping = 0.0
    } else {
        shipping = 5.99
    }

    log.info("Total: \(total), Shipping: \(shipping)")
}
```

## for Loops

Iterate over lists:

```fr
fn processItems: (items: list) => {
    for item in items {
        log.debug("Processing: \(item)")
    }
}

fn printNames: (users: list) => {
    for user in users {
        log.info("User: \(user.name) (\(user.email))")
    }
}

fn calculateTotal: (prices: list) => {
    :var mut total = 0.0
    for price in prices {
        total = total + price
    }
    return total
}
```

## switch/case Statements

```fr
fn handleStatus: (status: string) => {
    switch status {
        case "active" => {
            log.info("Active user")
            navigate("/dashboard")
        }
        case "inactive" => {
            log.warn("Inactive user")
            navigate("/reactivate")
        }
        case "banned" => {
            log.error("Banned user")
            navigate("/support")
        }
        default => {
            log.warn("Unknown status: \(status)")
        }
    }
}
```

### switch with Enum Values

```fr
:enum Status { Active  Inactive  Pending }

fn getStatusColor: (status: Status) => {
    switch status {
        case Status.Active => { return "#28A745" }
        case Status.Inactive => { return "#6C757D" }
        case Status.Pending => { return "#FFC107" }
    }
}
```

## try/catch

```fr
fn safeFetch: async (url: string) => {
    try {
        :var result = wait:fetch(url, { method: "GET" })
        if result != null {
            UserStore.data = result
        }
    } catch (err) {
        UserStore.error = "Failed: \(err)"
        log.error("Fetch error: \(err)")
    } finally {
        UserStore.is_loading = false
    }
}
```

## Data-Bound Lists

In UI, use the `list` component with `data` and `build`:

```fr
list: {
    data: UserStore.users
    build: (user) => {
        UserCard: { name: user.name  email: user.email }
    }
}

list: {
    data: UserStore.items
    build: (item) => {
        text: { content: item.name  styles: { padding: 8dp } }
    }
}
```

## Combined Example

```fr
fn renderDashboard: () => {
    if UserStore.is_loading {
        skeleton: { styles: { width: 100%  height: 80dp } }
    } else if UserStore.error != "" {
        text: {
            content: "Error: \(UserStore.error)"
            styles: { color: "#FF0000" }
        }
    } else {
        list: {
            data: UserStore.users
            build: (user) => {
                UserCard: {
                    name: user.name
                    email: user.email
                    show_if: user.is_active == true
                }
            }
        }
    }
}
```

## Syntax Reference

| Construct | Syntax |
|-----------|--------|
| show_if | `show_if: <expression>` |
| if/else | `if <expr> { ... } else { ... }` |
| for | `for <var> in <expr> { ... }` |
| switch | `switch <expr> { case <val> => { ... } default => { ... } }` |
| try/catch | `try { ... } catch (<var>) { ... } finally { ... }` |
