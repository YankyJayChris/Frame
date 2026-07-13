# @/ Path Aliases

Frame supports `@/` path aliases for imports, allowing you to reference files relative to a configured project root instead of using deep relative paths.

## Configuration

Path aliases are configured in `frame.config.json` using the `paths` field:

```json
{
  "paths": {
    "@": "./src"
  }
}
```

## Usage

Once configured, you can import files using the `@/` prefix instead of relative paths:

```fr
// Instead of:
import { MyComponent } "../../components/MyComponent"

// You can write:
import { MyComponent } "@/components/MyComponent"
```

## Resolution Rules

The `@/` alias resolves as follows:

1. The alias key `@` maps to the configured path (e.g., `./src`)
2. The path is resolved relative to the **project root** (the directory containing `frame.config.json`)
3. So `@/components/MyComponent` → `<project_root>/src/components/MyComponent.fr`

### Project Root Detection

If no `frame.config.json` is found, the system walks up from the current file's directory looking for:

1. A directory containing `frame.config.json`
2. A directory containing `src/project.fr`

The first directory found is used as the project root.

### Resolution Order

```
@/components/MyComponent.fr
    ↓
project root + /src + /components/MyComponent.fr
    ↓
<project_root>/src/components/MyComponent.fr
```

## Fallback Behavior

If the `paths` field is not configured in `frame.config.json`, the system defaults to `"./src"`:

```json
{
  "paths": {
    "@": "./src"
  }
}
```

This is the implicit default — you only need to add the `paths` field if you want a different alias target.

## Works with All Import Types

The `@/` prefix works with all import types:

```fr
// Component imports
import { MyComponent } "@/components/MyComponent"

// Function imports
import { loadUser, saveData } "@/controllers/UserController"

// Store imports
import { UserStore } "@/models/UserStore"

// Object type imports
import { User } "@/models/User"

// Enum imports
import { Status } "@/models/Status"

// Re-exports
export { MyComponent } "@/components/MyComponent"
```

## Example

Given this project structure:

```
myApp/
├── frame.config.json          # paths: { "@": "./src" }
├── src/
│   ├── project.fr
│   ├── views/
│   │   ├── pages/
│   │   │   └── Home.fr        # <-- current file
│   │   └── components/
│   │       └── UserCard.fr    # <-- target file
│   ├── models/
│   │   └── UserStore.fr
│   └── controllers/
│       └── UserController.fr
```

From `Home.fr`, instead of:

```fr
import { UserCard } "../components/UserCard"
import { UserStore } "../../models/UserStore"
import { loadUsers } "../../controllers/UserController"
```

You can write:

```fr
import { UserCard } "@/views/components/UserCard"
import { UserStore } "@/models/UserStore"
import { loadUsers } "@/controllers/UserController"
```
