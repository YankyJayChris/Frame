# HTTP Requests (Fetch)

Frame uses `wait:fetch()` for making HTTP requests. The `wait:` prefix is required because fetch is an async operation.

The options block accepts exactly four keys: `method`, `headers`, `body`, and `timeout`. All other data belongs inside `body`.

## GET Request

```fr
fn loadUsers: async () => {
    result = wait:fetch("/api/users", { method: "GET" })
    if result != null {
        UserStore.users = result
    }
}
```

## POST Request with Body

```fr
fn createUser: async () => {
    result = wait:fetch("/api/users", {
        method: "POST"
        body: {
            name: "Alice"
            email: "alice@example.com"
        }
    })
    if result != null {
        log.info("User created: \(result)")
    }
}
```

## PUT Request

```fr
fn updateUser: async (id: string) => {
    result = wait:fetch("/api/users/$id", {
        method: "PUT"
        body: {
            name: "Alice Updated"
            email: "alice@example.com"
        }
    })
}
```

## DELETE Request

```fr
fn deleteUser: async (id: string) => {
    result = wait:fetch("/api/users/$id", { method: "DELETE" })
    if result != null {
        log.info("User deleted")
    }
}
```

## Request with Headers

```fr
fn fetchWithAuth: async (token: string) => {
    result = wait:fetch("/api/protected", {
        method: "GET"
        headers: {
            Authorization: "Bearer \(token)"
            Content-Type: "application/json"
            Accept: "application/json"
        }
    })
}
```

## Request with Timeout

```fr
fn fetchWithTimeout: async () => {
    result = wait:fetch("/api/data", {
        method: "GET"
        timeout: 10000
    })
}
```

## POST with Headers and Body

```fr
fn login: async (username: string, password: string) => {
    result = wait:fetch("/api/auth/login", {
        method: "POST"
        headers: {
            Content-Type: "application/json"
        }
        body: {
            username: username
            password: password
        }
    })
    if result != null {
        AuthStore.token = result.token
    }
}
```

## Chaining with .then and .catch

```fr
fn fetchData: async () => {
    result = wait:fetch("/api/data", { method: "GET" })
        .then: (data) => {
            log.info("Got: \(data)")
            return data
        }
        .catch: (err) => {
            log.error("Failed: \(err)")
            return null
        }

    if result != null {
        UserStore.data = result
    }
}
```

## Full Example with Error Handling

```fr
fn loadProfile: async (userId: string) => {
    UserStore.is_loading = true
    UserStore.error = ""

    try {
        result = wait:fetch("/api/users/$userId", {
            method: "GET"
            headers: {
                Authorization: "Bearer \(UserStore.token)"
            }
            timeout: 5000
        })

        if result != null {
            UserStore.user = result
        } else {
            UserStore.error = "No data returned"
        }
    } catch (err) {
        UserStore.error = "Failed to load: \(err)"
        log.error("Profile load error: \(err)")
    }

    UserStore.is_loading = false
}
```

## Fetch API Reference

```fr
// GET — minimal
result = wait:fetch("/api/data", { method: "GET" })

// POST with body
result = wait:fetch("/api/data", {
    method: "POST"
    body: { name: "Alice"  email: "alice@example.com" }
})

// Full options
result = wait:fetch("/api/data", {
    method: "POST"
    headers: {
        Authorization: "Bearer \(token)"
        Content-Type: "application/json"
    }
    body: {
        name: "Alice"
        role: "admin"
    }
    timeout: 5000
})

// With chaining
result = wait:fetch("/api/data", { method: "GET" })
    .then: (res) => { return res }
    .catch: (err) => { return null }
```

## Fetch Options

| Option         | Type     | Default   | Description                              |
|----------------|----------|-----------|------------------------------------------|
| `method`       | string   | `"GET"`   | HTTP method: `"GET"`, `"POST"`, `"PUT"`, `"DELETE"` |
| `headers`      | object   | `{}`      | Key-value pairs added to request headers |
| `body`         | object   | `null`    | Request payload — format depends on `content_type` |
| `content_type` | string   | (none)    | Shorthand for `Content-Type` header — drives body serialization |
| `timeout`      | int (ms) | `30000`   | Request timeout in milliseconds          |

> **Important:** Only `method`, `headers`, `body`, `content_type`, and `timeout` are valid inside the options block.
> Any data you want to send must go inside `body: { ... }`.

## Body Serialization

The `content_type` (or `Content-Type` inside `headers:`) drives how `body:` is serialized:

| `content_type` value | Body encoding | Android | iOS |
|----------------------|--------------|---------|-----|
| `"application/json"` (default) | JSON | `Gson().toJson(body)` | `JSONSerialization` |
| `"application/x-www-form-urlencoded"` | URL-encoded form | `FormBody.Builder` | `key=value&...` string |
| `"multipart/form-data"` | Multipart | `MultipartBody.Builder` | `URLSession uploadTask` |

### JSON (default)

```fr
result = wait:fetch("/api/users", {
    method: "POST"
    body: {
        name: "Alice"
        email: "alice@example.com"
    }
})
```

### Form-encoded

```fr
result = wait:fetch("/api/auth", {
    method: "POST"
    content_type: "application/x-www-form-urlencoded"
    body: {
        username: userInput
        password: passInput
    }
})
```

### Explicit Content-Type via headers

```fr
result = wait:fetch("/api/upload", {
    method: "POST"
    headers: {
        Content-Type: "application/x-www-form-urlencoded"
        Authorization: "Bearer \(token)"
    }
    body: {
        file_name: "photo.jpg"
        data: base64Data
    }
})
```

> `content_type:` is a shorthand for setting `Content-Type` in `headers:`. If you need to set both `Content-Type` and other headers, use `headers:` directly.

## Platform Notes

| Platform | Networking Library |
|----------|-------------------|
| Android  | OkHttp + Gson (JSON) / FormBody (form-encoded) / MultipartBody (multipart) |
| iOS      | URLSession + JSONSerialization (JSON) / string encoding (form-encoded) |
