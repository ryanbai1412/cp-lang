# 4b. Developer tools: formatter, linter, profiler, coverage, bench, docs, versions

Status: proposed. The user asked for a formatter and linter and for "what
else is in the usual toolchains"; this is the answer.

## Formatter: `cp fmt`

- One canonical style with **no options**, as with `gofmt`: 4-space
  indentation, 100 columns, the syntax in 17-syntax.md.
- Works on the lossless syntax tree, so comments and blank-line grouping are
  kept.
- Guarantees, tested on the whole corpus: it never changes the token
  sequence (beyond whitespace and trailing commas), and formatting twice
  gives the same result as formatting once.
- Format-on-save and range formatting through the LSP; `cp fmt --check` for
  scripts.

## Linter: `cp lint`

Checks for code that is legal but probably wrong, slow or unidiomatic.
Lints never block a build. Each has a stable code, a default level, a
message that says what to write instead, and an autofix where possible.

```
warning[L0412]: `Map<V, Int>` keyed by a domain
  --> d.cp:7:11
help: `[V]?Int` uses 8 bytes per vertex and has no lookup cost
```

The initial set:

- `v.ord + 1` used to build a position.
- A domain-keyed `Map` or `Set`.
- A `var` that's never reassigned.
- A struct field that's never read.
- `==` on floats.
- A `@memo` whose key space exceeds the memory budget at the input bounds.
- Shadowing a top-level `let`.
- A `List` index that couldn't be proven in range, inside a hot loop.

Levels can be set in `cp.toml` (`[lints]`) and per item with
`@allow(name)`. Lints run incrementally in the editor.

## Profiler: `cp profile d big.in`

- Time per line and per part, plus allocation sizes, shown inline in the
  editor and as an HTML flame graph.
- The native build uses sampling (`perf`, where available). The interpreter
  counts statements exactly, which is more precise and works on small
  inputs.
- It puts the cost estimate next to the measured counts, so a
  wrong estimate is easy to see.

## Coverage: `cp cover`

- Which lines and branches your samples, saved tests and stress runs
  actually ran, shown as editor gutter colors.
- `cp stress --until-covered` keeps generating until every branch of the
  solution has run at least once, or reports the ones it never reached.

## Benchmark: `cp bench d`

- Runs the native build on generated max-size inputs (from the input types)
  and on the worst shapes (paths, stars, all-equal values), and reports time
  and memory against the problem's limits.
- `cp bench --compare HEAD~1` checks for regressions in your library.

## Documentation generator: `cp doc`

- HTML docs from `///` comments for your library and for the stdlib, with
  types, complexity from `@cost` annotations, examples that are run as tests,
  and source links.
- The documentation site uses the same generator, so stdlib docs can't drift
  from the code.

## Refactorings, through the LSP

- **Rename** across files.
- **Extract and inline** a fn.
- **Turn `Int` indexing into a domain:** introduce `domain V` and update the
  arrays and loops that use it.
- **Convert between table and loop forms.**
- **Add a brute for this fn:** creates a `@brute` stub with the same
  signature.

## REPL: `cp repl d i2`

As proposed in 18-key-features.md: load a test, then evaluate fns and
expressions against it, with `:trace` and `:snap`.

## Version manager

- `cp self install 0.4`, `cp self use 0.4`, and a per-project pin in
  `cp.toml`.
- Old files carry version stamps (doc 4, section 5), so `cp build` on an
  old file uses the version it was written with, or offers
  `cp fix --upgrade`.
- Upgrades between versions are automated with `cp fix --upgrade`, the way
  Rust editions work.

## Not planned

- **Sanitizers:** the debug interpreter already checks bounds, overflow,
  ranges and uninitialized reads on every run.
- **Dependency auditing:** there's no public package registry.
