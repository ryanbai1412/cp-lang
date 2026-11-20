This course takes you from your first working program to the full CP Lang v2.2 model: structural range arrays, nominal domains, `.l`/`.r` boundaries, grids with `p.adj4`/`p.adj8`, value-semantic `Str`, lexical borrowing with `lends`, tropical `?Int`, bare operators / `Monoid(id, op)` / `Action`, `@memo`, `Rooted<V>`, and `@brute`/`@checker`/`@interactor` verification.

Each chapter teaches one idea from first principles, walks through a concrete program, and ends with checked exercises whose tests run locally in WebAssembly inside this page.

```course
```

## How the interactive exercises work

Every chapter includes a runnable worked program and two or three exercises. Code exercises compile and execute your edited CP Lang source through the real Rust `cpl` compiler and interpreter bundled as WebAssembly right inside the page (no network required). Conceptual exercises check your prediction about types, bounds, ownership, or runtime behavior with immediate explanations.

Progress is stored only in `localStorage` on your device. No account, server, or network access is required.

## Two ways to read

- **Learn end to end:** start at [Chapter 1](course-first-program.html) and work forward. Each chapter assumes only the previous ones.
- **Look up a single topic:** use the [one-page cheat sheet](cheat-sheet.html), the [reference guide](index.html), or the [offline playground](playground.html) to experiment directly.
