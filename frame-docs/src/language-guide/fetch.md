# HTTP Requests (Fetch)

Frame uses `wait:fetch()` for making HTTP requests. The `wait:` prefix is required because fetch is an async operation.

The options block accepts exactly four keys: `method`, `headers`, `body`, and `timeout`.
Data you want to send goes inside `body: { ... }`. The `Content-Type` header goes inside `headers: { ... }`.

## GET Request

```fr
fn loadUsers: async () => {
    result = wait:fetch("/api/users", { method: "GET" })
    if result != null {
        UserStore.users = result
    }
}
```

## POST with JSON Body

```fr
fn createUser: async () => {
    result = wait:fetch("/api/users", {
        method: "POST"
        headers: {
            Content-Type: "application/json"
        }
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

## POST as Form-Encoded

```fr
fn loginWithForm: async () => {
    result = wait:fetch("/api/auth/login", {
        method: "POST"
        headers: {
            Content-Type: "application/x-www-form-urlencoded"
        }
        body: {
            username: userInput
            password: passInput
        }
    })
}
```

## POST as Multipart

```fr
fn uploadFile: async () => {
    result = wait:fetch("/api/upload", {
        method: "POST"
        headers: {
            Content-Type: "multipart/form-data"
        }
        body: {
            file: selectedFile
            caption: captionText
        }
    })
}
```

## PUT Request

```fr
fn updateUser: async (id: string) => {
    result = wait:fetch("/api/users/$id", {
        method: "PUT"
        headers: {
            Content-Type: "application/json"
        }
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

## With Auth Header

```fr
fn fetchWithAuth: async (token: string) => {
    result = wait:fetch("/api/protected", {
        method: "GET"
        headers: {
            Authorization: "Bearer \(token)"
            Accept: "application/json"
        }
    })
}
```

## With Timeout

```fr
fn fetchWithTimeout: async () => {
    result = wait:fetch("/api/data", {
        method: "GET"
        timeout: 10000
    })
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
                Accept: "application/json"
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
// Minimal GET
result = wait:fetch("/api/data", { method: "GET" })

// POST with JSON body
result = wait:fetch("/api/data", {
    method: "POST"
    headers: {
        Content-Type: "application/json"
    }
    body: {
        name: "Alice"
        email: "alice@example.com"
    }
})

// POST form-encoded
result = wait:fetch("/api/login", {
    method: "POST"
    headers: {
        Content-Type: "application/x-www-form-urlencoded"
    }
    body: {
        username: "alice"
        password: "secret"
    }
})

// Full options
result = wait:fetch("/api/data", {
    method: "POST"
    headers: {
        Authorization: "Bearer \(token)"
        Content-Type: "application/json"
        Accept: "application/json"
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

| Option    | Type     | Default   | Description                              |
|-----------|----------|-----------|------------------------------------------|
| `method`  | string   | `"GET"`   | HTTP method: `"GET"`, `"POST"`, `"PUT"`, `"DELETE"` |
| `headers` | object   | `{}`      | Key-value request headers — set `Content-Type` here |
| `body`    | object   | `null`    | Request payload — serialized based on `Content-Type` |
| `timeout` | int (ms) | `30000`   | Request timeout in milliseconds          |

> **Rules:**
> - Only `method`, `headers`, `body`, and `timeout` are valid inside the options block
> - `Content-Type` belongs inside `headers: { ... }`
> - Request data belongs inside `body: { ... }`

## Body Serialization

The `Content-Type` header (set in `headers:`) drives how `body:` is serialized:

| `Content-Type` | Body encoding | Android | iOS |
|----------------|--------------|---------|-----|
| `"application/json"` (default) | JSON | `Gson().toJson(body)` | `JSONSerialization` |
| `"application/x-www-form-urlencoded"` | URL-encoded | `FormBody.Builder` | `key=value&...` |
| `"multipart/form-data"` | Multipart | `MultipartBody.Builder` | `URLSession uploadTask` |

> When `body:` is provided but no `Content-Type` header is set, Frame defaults to `application/json`.

## Platform Notes

| Platform | Networking Library |
|----------|-------------------|
| Android  | OkHttp + Gson (JSON) / FormBody (form-encoded) / MultipartBody (multipart) |
| iOS      | URLSession + JSONSerialization (JSON) / string encoding (form-encoded) |
