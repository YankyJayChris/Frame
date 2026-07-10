# blog-app

A Frame cross-platform mobile app using **MVC**.

## Project Structure

```
src/
  models/         # :obj types + store state
  views/          # Pages and components
  controllers/    # Business logic functions
```

## Key Concepts

- **`:obj`** — typed data model (entity / API shape). Compiles to a Kotlin `data class` and Swift `struct`.
- **`:store`** — reactive global state. Compiles to a Kotlin `ViewModel` and Swift `ObservableObject`.

## Commands

```bash
frame check          # verify build environment
frame build          # compile .fr files
frame test           # run test suites
frame deploy android # generate Android project
frame deploy ios     # generate iOS project
frame preview        # hot-reload dev server
```
