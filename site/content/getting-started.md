You can try CP Lang immediately with zero setup in the [browser playground](playground.html), or install the local `cpl` CLI to run programs natively, emit standalone C++17 for online judges, and enable editor support (`cpl lsp`).

## Install from source

The `cpl` compiler is written in Rust and compiles CP Lang programs to standard C++17 (compatible with any `g++` or `clang++` supporting `-std=c++17`).

```sh
git clone https://github.com/ryanbai1412/cp-lang.git
cd cp-lang
cargo install --path crates/cpl-cli
```

Verify the installation:

```sh
cpl --version
cpl --help
```

## Write, run, and compile your first program

Save this example as `sum.cp`:

```example sum
```

Check types, run directly with the local interpreter, or compile to a standalone C++17 submission:

```sh
cpl check sum.cp
printf '5\n3 1 4 1 5\n' | cpl run sum.cp
cpl build sum.cp -o sum.cpp
g++ -std=c++17 -O2 sum.cpp -o sum
printf '5\n3 1 4 1 5\n' | ./sum
```

The output is `14`.

`cpl build` writes a single self-contained `.cpp` file with your original `.cp` source preserved as a comment header at the top, unused library code tree-shaken away, and `@brute` / `@checker` / `@interactor` blocks stripped so the file is clean and ready to upload to any judge.

## Editor support (`cpl lsp`)

The `cpl` CLI ships with a built-in Language Server Protocol server (`cpl lsp`) powered by the same incremental frontend as `cpl check`. Point your editor's LSP client at `cpl lsp` for real-time diagnostics, hover types, and completion while writing `.cp` files.

## What works today

- **Available now in `cpl` and the browser WebAssembly playground:** `cpl check`, `cpl run`, `cpl build` (readable C++17 emission), `cpl lsp`, live execution traces (`Trace` button in the playground and [Trace debugger](trace.html)), `@!multitest`, top-level `return`, nominal domains, `.l`/`.r` boundaries, multi-dimensional arrays, enums and `match`, Tropical Optionals (`?T`, `min=`, `max=`, `??`, `??=`), cached `@memo` DP, `lends mut` projections, and standard library modules including `List`, `MinHeap`, `Fenwick`, `SegTree`, `LazySegTree`, `Dsu`, `Mod<P>`, and `Comb<M>`.
- **In progress for upcoming CLI releases:** automated `cpl stress` with structure-aware shrinking for `@brute` / `@checker` / `@interactor`, `cpl bench` worst-case judge timing and RSS checks, `cpl new` constraint scaffolding, and `cpl test` / `cpl doc` for local libraries.

Next, work through the [interactive course](course.html), take the [language tour](tour.html), or jump straight to the [C++17 migration guide](cpp-guide.html).
