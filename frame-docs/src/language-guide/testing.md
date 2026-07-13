# Testing

Frame includes a built-in testing framework using `describe:`, `it:`, `expect:`, and `mock:`.

## Basic Test Suite

```fr
describe: "UserStore" => {
    it: "is_loading starts false" => {
        expect: false .toBeFalse:()
    }

    it: "error starts empty" => {
        expect: "" .toBe: ""
    }
}
```

## Testing Functions

```fr
fn double: (x: int) => {
    return x * 2
}

describe: "Math" => {
    it: "doubles positive numbers" => {
        expect: double(5) .toBe: 10
    }

    it: "doubles zero" => {
        expect: double(0) .toBe: 0
    }

    it: "doubles negative numbers" => {
        expect: double(-3) .toBe: -6
    }
}
```

## Testing with Mocks

Mock HTTP responses for async tests:

```fr
describe: "API" => {
    it: "fetches user data" => {
        mock: {
            url: "/api/users/1"
            response: { id: "1"  name: "Jane"  email: "jane@test.com" }
            status: 200
        }

        :var result = wait:fetch("/api/users/1", { method: "GET" })
        expect: result.name .toBe: "Jane"
    }

    it: "handles 404 gracefully" => {
        mock: {
            url: "/api/users/999"
            response: { error: "Not found" }
            status: 404
        }

        :var result = wait:fetch("/api/users/999", { method: "GET" })
        expect: result.error .toBe: "Not found"
    }
}
```

## Testing Component Behavior

```fr
describe: "UserCard" => {
    it: "renders user name" => {
        mock: { url: "/api/user"  response: { name: "Alice" }  status: 200 }

        :var result = wait:fetch("/api/user", { method: "GET" })
        expect: result.name .toBe: "Alice"
    }

    it: "shows loading state" => {
        :var isLoading = true
        expect: isLoading .toBeTrue:()
    }
}
```

## Multiple Test Cases

```fr
import { UserStore } "@/stores/user_store"

describe: "UserStore Actions" => {
    it: "increments count" => {
        UserStore.count = 0
        UserStore.increment()
        expect: UserStore.count .toBe: 1
    }

    it: "toggles dark mode from false" => {
        UserStore.dark_mode = false
        UserStore.toggleDarkMode()
        expect: UserStore.dark_mode .toBeTrue:()
    }

    it: "toggles dark mode from true" => {
        UserStore.dark_mode = true
        UserStore.toggleDarkMode()
        expect: UserStore.dark_mode .toBeFalse:()
    }

    it: "resets state" => {
        UserStore.count = 42
        UserStore.user_name = "Test"
        UserStore.reset()
        expect: UserStore.count .toBe: 0
        expect: UserStore.user_name .toBe: ""
    }
}
```

## Matchers Reference

| Matcher | Usage | Description |
|---------|-------|-------------|
| `.toBe:` | `expect: value .toBe: expected` | Strict equality |
| `.toEqual:` | `expect: value .toEqual: expected` | Deep equality |
| `.toContain:` | `expect: list .toContain: item` | Contains item |
| `.toBeNull:` | `expect: value .toBeNull:()` | Is null |
| `.toBeTrue:` | `expect: value .toBeTrue:()` | Is true |
| `.toBeFalse:` | `expect: value .toBeFalse:()` | Is false |
| `.toThrow:` | `expect: fn .toThrow:()` | Throws error |

## Running Tests

```bash
# Run all tests
frame test

# Run tests matching a filter
frame test --filter UserStore

# Run with coverage
frame test --coverage
```

Tests are discovered from `*.test.fr` files in the project.

## Test File Structure

```
myApp/src/tests/
├── user_store.test.fr
├── api.test.fr
├── components/
│   ├── user_card.test.fr
│   └── header.test.fr
└── utils/
    └── helpers.test.fr
```

## Test Syntax Reference

```fr
describe: "Suite Name" => {
    it: "test case description" => {
        // Setup
        mock: { url: "/api/..."  response: { ... }  status: 200 }

        // Execute
        :var result = someFunction()

        // Assert
        expect: result .matcher: expected
    }
}
```
