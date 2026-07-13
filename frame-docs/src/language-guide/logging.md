# Logging

Frame provides built-in logging functions at multiple levels. Use the `log.*` methods for structured logging and `util.print()` for simple console output.

## Basic Print

```fr
util.print("Hello, World!")
util.print("User logged in: \(user.name)")
util.print("Count: \(count)")
```

## Log Levels

```fr
log.verbose("Detailed debug: \(state)")
log.debug("Button pressed at \(x), \(y)")
log.info("App started")
log.warn("Cache miss for user \(userId)")
log.error("Failed to load: \(errorMessage)")
```

## Logging with Interpolation

```fr
fn processUser: (userId: string) => {
    log.info("Processing user: \(userId)")

    if UserStore.is_loading {
        log.debug("User \(userId) is still loading")
    }

    if UserStore.error != "" {
        log.error("Error for user \(userId): \(UserStore.error)")
    }
}
```

## Logging in Async Functions

```fr
fn fetchData: async () => {
    log.info("Starting data fetch")
    result = wait:fetch("/api/data", { method: "GET" })
    if result != null {
        log.debug("Data received: \(result)")
    } else {
        log.warn("No data returned")
    }
    log.info("Fetch complete")
}
```

## Platform Behavior

| Level   | Android      | iOS                          |
|---------|--------------|------------------------------|
| VERBOSE | `Log.v(...)` | `os_log(.debug, ...)`        |
| DEBUG   | `Log.d(...)` | `os_log(.debug, ...)`        |
| INFO    | `Log.i(...)` | `os_log(.info, ...)`         |
| WARN    | `Log.w(...)` | `os_log(.default, ...)`      |
| ERROR   | `Log.e(...)` | `os_log(.error, ...)`        |

## Logging Guidelines

- Use `log.info` for general application flow events
- Use `log.debug` for development-time diagnostics
- Use `log.warn` for recoverable issues
- Use `log.error` for failures and exceptions
- Use `log.verbose` for detailed trace information
- Avoid logging sensitive information (passwords, tokens)
- Logs are stripped from release builds in some configurations
