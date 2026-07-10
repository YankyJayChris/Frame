# profile

A Frame cross-platform mobile app using **Clean Architecture**.

## Project Structure

```
src/
  domain/         # :obj entities, use cases, repository interfaces
  data/           # Store implementations, API models
  presentation/   # Pages, components, stores
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
