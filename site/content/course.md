This course takes you from your first working program to the full CP Lang 0.2.2 model: `@!multitest` and top-level `return`, nominal domains and `.l`/`.r` boundaries, first-principles `Index` and slices, `lends` and lexical borrowing without lifetimes, Tropical Optionals (`?T`, `min=`, `max=`, `??=`) and cached `@memo` DP, algebraic `Monoid`/`Action` segment trees, and co-located `@brute`/`@checker`/`@interactor` verification.

Each chapter teaches one idea from first principles, walks through concrete programs, and checks your understanding with exercises that execute locally in WebAssembly inside this page.

```course
```

## How the interactive exercises work

Every chapter pairs worked programs with knowledge checks and output-prediction exercises. Output exercises compile and run real CP Lang source through the Rust `cpl` compiler and interpreter bundled as WebAssembly (no network or mock runtime), and include an editable sandbox with C++17 codegen and cursor type hover.

Progress is stored only in `localStorage` on your device. No account, server, or network access is required.

## Two ways to read

- **Learn end to end:** start at [Chapter 1](course-first-program.html) and work forward. Each chapter builds only on the previous ones.
- **Look up a single topic:** use the [one-page cheat sheet](cheat-sheet.html), the [reference guide](index.html), or the [offline playground](playground.html) to experiment directly.
