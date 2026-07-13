# Named Arguments

Frame supports named arguments in function calls, component props, and event handlers.

## Function Calls with Named Arguments

```fr
fn greet: (name: string, greeting: string = "Hello") => {
    log.info("\(greeting), \(name)!")
}

// Positional
greet("Alice")

// Named
greet(name: "Alice")
greet(name: "Bob", greeting: "Hi")
```

## Mixed Positional and Named

```fr
fn configure: (host: string, port: int = 8080, ssl: bool = false) => {
    log.info("\(host):\(port) ssl=\(ssl)")
}

// First positional, rest named
configure("example.com", port: 443, ssl: true)
configure("localhost", ssl: false)
```

## Named Arguments in Async Calls

```fr
fn loadUser: async (id: string, includePosts: bool = false) => {
    result = wait:fetch("/api/users/\(id)", { method: "GET" })
}

// Named
wait:UserStore.load(id: "42")
wait:UserStore.load(id: "42", includePosts: true)
```

## Named Arguments in Component Props

Components receive all props as named arguments:

```fr
UserCard: {
    name: "Alice Johnson"
    email: "alice@example.com"
    bio: "Developer"
    avatar: "https://i.pravatar.cc/80"
}
```

## Named Arguments in Event Handlers

```fr
button: {
    content: "Submit"
    on_click: submitForm()
}

input: {
    value: email
    on_change: validateEmail(value: email)
    on_submit: handleSubmit(formData: state.formData)
}
```

## Named Arguments in Style Blocks

Style properties use named argument syntax:

```fr
container: {
    styles: {
        width: 100%
        height: 200dp
        background: "#FFFFFF"
        padding: 16dp
    }
}
```

## Named Arguments in Navigation

```fr
navigate("/home", clear_stack: true)
navigate("/detail", replace: true, transition: "slide_up")
navigate("/profile", single_top: true)
```

## Named Arguments in Animations

```fr
animate: {
    property: opacity
    from: 0
    to: 1
    duration: 300ms
    easing: ease_in_out
}
```

## Named Arguments in Plugin Calls

```fr
plugin: {
    name: "frame-storage"
    method: "saveFile"
    params: {
        filename: "data.txt"
        content: "Hello"
    }
}
```

## Rules

- Named arguments use the format `name: value`.
- Positional arguments must come before named arguments in a call.
- Named arguments can be in any order.
- Default parameter values are used when a named argument is omitted.
