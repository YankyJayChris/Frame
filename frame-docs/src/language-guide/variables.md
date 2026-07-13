# Variables and Constants

Frame provides three mechanisms for declaring named values: `:vars` design tokens, `const` compile-time constants, and `:var` local variables.

## Design Tokens (`:vars`)

Global design tokens are defined in a `:vars` block, typically at the top of `project.fr`. They are referenced with a `$` prefix.

```fr
:vars {
    $primary:   "#3584e4"
    $secondary: "#6C757D"
    $success:   "#28A745"
    $danger:    "#DC3545"
    $surface:   "#ffffff"
    $text:      "#333333"
    $radius:    "8dp"
    $spacing:   16dp
    $gap:       8dp
}
```

Referencing design tokens:

```fr
button: {
    content: "Submit"
    styles: {
        background: $primary
        color: "#FFFFFF"
        border_radius: $radius
        padding: $spacing
    }
}

text: {
    content: "Hello"
    styles: { color: $text }
}
```

## Predicate Variables (`$$`)

Predicate variables use `$$` prefix and reference the current component's props or state:

```fr
component HighlightedText: {
    props: {
        text: string
        highlight: string = ""
    }
    children: [
        text: {
            content: text
            styles: { color: $$highlight != "" ? $primary : $text }
        }
    ]
}
```

## Compile-Time Constants (`const`)

Constants are compile-time immutable values. They support `string`, `int`, `float`, and `bool` types.

```fr
const apiUrl = "https://api.example.com"
const maxItems = 100
const debugMode = true
const pi = 3.14159
const appName = "FrameApp"
const timeoutMs = 5000
const enableAnalytics = false
```

Constants are referenced by name:

```fr
fn loadData: async () => {
    result = wait:fetch(apiUrl, {
        method: "GET"
        timeout: timeoutMs
    })
}
```

## Local Variables (`:var`)

Inside function bodies, `:var` declares local variables with optional type annotation and mutability.

### Immutable Variables

```fr
fn process: () => {
    :var greeting = "Hello"          // inferred string, immutable
    :var count = 42                  // inferred int
    :var name: string = "World"      // explicit type
    :var items: list = [1, 2, 3]     // typed list
}
```

### Mutable Variables

Use the `mut` keyword to allow reassignment:

```fr
fn process: () => {
    :var mut count: int = 0
    :var mut total: float = 0.0
    :var mut name = "Alice"

    count = count + 1
    total = total + 1.5
    name = "Bob"
}
```

### Variable Declaration Rules

```
:var name = expr              // inferred type, immutable
:var name: type               // explicit type, no initializer
:var name: type = expr        // explicit type with initializer
:var mut name = expr          // inferred type, mutable
:var mut name: type = expr    // explicit type, mutable
```

## Variable Lifetime

| Kind     | Scope                     | Mutability |
|----------|---------------------------|------------|
| `:vars`  | Global — entire project    | Immutable  |
| `const`  | Global — entire project    | Immutable  |
| `:var`   | Local — enclosing function | Default immutable, `mut` for mutable |

## Variable References in Expressions

Variables are referenced by name in expressions:

```fr
fn calculate: (x: int, y: int) => {
    :var sum = x + y
    :var mut result = sum * 2
    result = result + 1
    return result
}
```

Store and state references use dot notation:

```fr
// Store field
text: { content: UserStore.user_name }

// Local page/component state
text: { content: state.count }
```
