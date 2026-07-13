# String Interpolation

Frame supports two equivalent syntaxes for embedding expressions inside strings: `\(expr)` and `${expr}`.

## Paren Interpolation `\(expr)`

```fr
:var name = "Alice"
:var age = 30

let greeting = "Hello, \(name)! You are \(age) years old."
// Result: "Hello, Alice! You are 30 years old."
```

## Brace Interpolation `${expr}`

```fr
:var userId = 42
:var page = 1

let url = "https://api.example.com/users/${userId}/posts?page=${page}"
// Result: "https://api.example.com/users/42/posts?page=1"
```

## Mixed Interpolation

Both syntaxes can be combined in the same string:

```fr
let message = "User \(name) has ${count} items and \(status) status."
```

## Interpolation in Component Props

```fr
text: {
    content: "Hello \(name), you have \(count) messages"
}

text: {
    content: "Path: ${basePath}/\(fileName).txt"
}
```

## Interpolation in Fetch URLs

```fr
fn loadUser: async (id: string) => {
    result = wait:fetch("/api/users/\(id)", { method: "GET" })
}

fn loadPost: async (userId: string, postId: string) => {
    result = wait:fetch("/api/users/${userId}/posts/\(postId)", { method: "GET" })
}
```

## Interpolation Expressions

Any expression can be used inside interpolation:

```fr
text: { content: "Total: \(price * quantity) USD" }
text: { content: "Status: ${isActive ? "Active" : "Inactive"}" }
text: { content: "Score: \(user.score ?? 0)" }
text: { content: "Name: \(user.firstName + " " + user.lastName)" }
```

## Interpolation in String Constants

```fr
const urlTemplate = "/api/users/\(userId)"
const greeting = "Welcome, ${name}!"
```

## Escaping

To include a literal `\(` or `${` in a string, escape them:

```fr
// In Frame, backslash escapes the parenthesis
let literal = "Use \\\\(escaped) for interpolation"
```

## Platform Behavior

Interpolated strings compile to native string formatting on each platform:

| Platform | Output                       |
|----------|------------------------------|
| Android  | Kotlin string templates      |
| iOS      | Swift string interpolation   |
