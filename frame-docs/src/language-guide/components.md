# Components

Components are reusable building blocks with typed `props`, `styles`, and `children`. They are defined with the `component` keyword and use PascalCase names.

## Basic Component

```fr
component Greeting: {
    props: {
        name: string = ""
    }
    children: [
        text: { content: "Hello, \(name)!"  styles: { font_size: 18sp } }
    ]
}
```

## Component with Required and Optional Props

```fr
component UserCard: {
    props: {
        name: string        // required — no default
        email: string       // required — no default
        avatar: string?     // optional (nullable)
        bio: string = ""    // optional — has default
    }
    styles: {
        border_radius: 8dp
        padding: 12dp
        margin_bottom: 8dp
        background: "#FFFFFF"
    }
    children: [
        column: {
            styles: { gap: 4dp  align: "center" }
            children: [
                avatar: { src: avatar  styles: { width: 48dp  height: 48dp }  show_if: avatar != null }
                text: { content: name  styles: { font_size: 16sp  font_weight: "bold" } }
                text: { content: email  styles: { font_size: 14sp  color: "#666" } }
                text: { content: bio  styles: { font_size: 14sp  font_style: "italic" }  show_if: bio != "" }
            ]
        }
    ]
}
```

## Component with Local State

```fr
component Counter: {
    props: {
        initialValue: int = 0
    }
    state: {
        count: int = 0
    }
    styles: {
        padding: 16dp
        align: "center"
    }
    children: [
        text: { content: "Count: \(state.count)"  styles: { font_size: 24sp } }
        row: {
            styles: { gap: 8dp }
            children: [
                button: { content: "-"  on_click: decrement() }
                button: { content: "+"  on_click: increment() }
            ]
        }
    ]

    fn increment: () => {
        state.count = state.count + 1
    }

    fn decrement: () => {
        state.count = state.count - 1
    }
}
```

## Component with Events

```fr
component TappableCard: {
    props: {
        title: string
        subtitle: string = ""
    }
    styles: {
        border_radius: 12dp
        padding: 16dp
        background: "#F8F9FA"
    }
    children: [
        column: {
            styles: { gap: 4dp }
            children: [
                text: { content: title  styles: { font_size: 16sp  font_weight: "bold" } }
                text: { content: subtitle  styles: { font_size: 14sp  color: "#666" }  show_if: subtitle != "" }
            ]
        }
    ]
    events: {
        on_click: handleTap()
    }

    fn handleTap: () => {
        log.info("Tapped card: \(title)")
    }
}
```

## Using a Component

```fr
// Import the component
import { UserCard } "@/components/UserCard"
import { Counter } "@/components/Counter"

page: {
    name: "Home"
    route: "/"
    children: [
        column: {
            styles: { padding: 16dp  gap: 12dp }
            children: [
                UserCard: {
                    name: "Alice Johnson"
                    email: "alice@example.com"
                    avatar: "https://i.pravatar.cc/80"
                    bio: "Full-stack developer"
                }
                Counter: {
                    initialValue: 5
                }
                TappableCard: {
                    title: "Settings"
                    subtitle: "Tap to configure"
                }
            ]
        }
    ]
}
```

## Conditional Rendering with `show_if`

```fr
UserCard: {
    name: UserStore.user_name
    email: UserStore.user_email
    bio: UserStore.user_bio
    show_if: UserStore.user_name != ""
}
```

## Prop Rules

- Required props use `name: type` with no default — they must always be passed.
- Optional props use `name: type = value` — they have a default value.
- Nullable props use `name: type?` — they accept `null`.
- Access props directly by name inside the component body.
- `show_if: expr` conditionally renders the component.

## Component Structure

| Section     | Keyword      | Description                           |
|-------------|--------------|---------------------------------------|
| Props       | `props:`     | Typed input parameters                |
| State       | `state:`     | Local mutable state fields            |
| Styles      | `styles:`    | Default style properties              |
| Children    | `children:`  | Component tree                        |
| Events      | `events:`    | Event handler declarations            |
| Functions   | `fn`         | Component-scoped functions            |
| Animations  | `animate:`   | Animation definitions                 |
