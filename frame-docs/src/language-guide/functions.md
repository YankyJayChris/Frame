# Functions

Functions are defined with the `fn` keyword followed by the function name, parameter list, and body.

## Basic Function

```fr
fn greet: (name: string) => {
    log.info("Hello, \(name)!")
}
```

## Function with Return Value

```fr
fn double: (x: int) => {
    return x * 2
}

fn add: (a: int, b: int) => {
    return a + b
}
```

## Async Functions

Use the `async` keyword to define an asynchronous function. Async functions must be called with the `wait:` prefix.

```fr
fn loadUserData: async (userId: string) => {
    result = wait:fetch("/api/users/\(userId)", { method: "GET" })
    if result != null {
        UserStore.user = result
    }
}
```

## Function with Default Parameters

```fr
fn greet: (name: string, greeting: string = "Hello") => {
    log.info("\(greeting), \(name)!")
}

fn fetchData: async (url: string, method: string = "GET") => {
    return wait:fetch(url, { method: method })
}
```

## Calling Functions

### Sync Calls

```fr
fn process: () => {
    :var result = double(5)     // result = 10
    greet(name: "Alice")        // named argument
    add(a: 1, b: 2)            // positional + named
}
```

### Async Calls (must use `wait:`)

```fr
fn processData: async () => {
    wait:loadUserData("42")
    :var data = wait:fetchData("/api/items")
    :var result = wait:fetchData("/api/items", method: "POST")
}
```

Calling an async function without `wait:` is a compile error.

## Function with Multiple Statements

```fr
fn processOrder: async (orderId: string) => {
    :var order = wait:fetchData("/api/orders/\(orderId)")
    if order == null {
        log.error("Order not found: \(orderId)")
        return
    }
    :var mut total = 0
    for item in order.items {
        total = total + item.price
    }
    log.info("Order \(orderId) total: \(total)")
}
```

## Named Arguments

```fr
fn configure: (host: string, port: int = 8080, ssl: bool = false) => {
    log.info("Connecting to \(host):\(port) (ssl=\(ssl))")
}

// All named
configure(host: "example.com", port: 443, ssl: true)

// Mixed positional + named
configure("example.com", ssl: true)

// Named with default
configure(host: "localhost")
```

## Function Scope

Functions defined at the top level of a file are globally accessible after import. Functions defined inside components are scoped to that component.

```fr
// Top-level function — accessible anywhere after import
fn formatDate: (timestamp: string) => {
    return date.format(date.parse(timestamp), "yyyy-MM-dd")
}

// Component function — scoped to component
component ProfileCard: {
    props: { name: string }

    fn formatName: (prefix: string) => {
        return "\(prefix) \(name)"
    }

    children: [
        text: { content: formatName("Dr.") }
    ]
}
```

## Return Statement

Use `return` to return a value from a function. Functions without a return value implicitly return `null`.

```fr
fn min: (a: int, b: int) => {
    if a < b {
        return a
    }
    return b
}
```

## Function Syntax Reference

```
fn <name>: [async] (<params>) => {
    <statements>
}
```

Each parameter follows: `name: type [= default_value]`
