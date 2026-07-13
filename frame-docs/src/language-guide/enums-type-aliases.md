# Enums and Type Aliases

## Enums (`:enum`)

Enums define a set of named variants. Variants can have optional associated string values.

### Enum Without Values

```fr
:enum Status {
    Active
    Inactive
    Pending
}

:enum Direction {
    North
    South
    East
    West
}

:enum Role {
    Admin
    User
    Moderator
    Guest
}
```

### Enum With String Values

```fr
:enum Color {
    Red   = "#FF0000"
    Green = "#00FF00"
    Blue  = "#0000FF"
}

:enum HttpStatus {
    Ok       = "200"
    NotFound = "404"
    Error    = "500"
}

:enum Theme {
    Light = "light_mode"
    Dark  = "dark_mode"
    System = "system_default"
}
```

### Using Enums

```fr
import { Status, Color } "@/enums/status"

fn getStatusLabel: (status: Status) => {
    switch status {
        case Status.Active => { return "User is active" }
        case Status.Inactive => { return "User is inactive" }
        case Status.Pending => { return "User is pending" }
    }
}

page: {
    name: "Settings"
    route: "/settings"
    children: [
        dropdown: {
            value: selectedTheme
            children: [
                text: { content: "Light" }
                text: { content: "Dark" }
                text: { content: "System" }
            ]
        }
    ]
}
```

### Enum Reference

| Aspect | Description |
|--------|-------------|
| Declaration | `:enum Name { Variant1 Variant2 }` |
| With values | `:enum Name { Key = "value" }` |
| Access | `EnumName.VariantName` |
| Compiles to | Kotlin `enum class` / Swift `enum` |

## Type Aliases (`:type`)

Type aliases create a new name for an existing type.

```fr
:type UserId = string
:type Score = int
:type JsonMap = object
:type ItemList = list
:type Callback = () => void
```

### Using Type Aliases

```fr
:type UserId = string
:type Age = int

page: {
    name: "Profile"
    route: "/profile/:userId"
    params: { userId: UserId }

    state: {
        userAge: Age = 0
    }
    children: [ ... ]
}

fn getUser: async (id: UserId) => {
    result = wait:fetch("/api/users/\(id)", { method: "GET" })
}
```

## Object Types (`:obj`)

Define structured data models with typed fields:

```fr
:obj User {
    id: string
    name: string
    email: string
    age: int?
    avatar: string?
}

:obj Address {
    street: string
    city: string
    zip: string
    country: string
}

:obj Post {
    id: string
    title: string
    content: string
    author: User
    comments: list
    published: bool
}
```

Optional fields are marked with `?` suffix.

### Using Object Types

```fr
fn parseUser: (data: object) => {
    :var user: User = from_json(data)
    log.info("User: \(user.name)")
}
```
