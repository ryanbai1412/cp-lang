CP Lang is a language for competitive programming that targets C++17. Keep the algorithms you know. Replace index arithmetic, sentinel values, and repeated bookkeeping with types that say what you mean.

Start with [your first program](getting-started.html), take the [language tour](tour.html), or go straight to [index spaces](indices.html).

```cp Prefix sums · the approved design
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()

var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]

fn range_sum(l: V, r: V) -> Int:
    pre[r.r] - pre[l.l]
```

An element has a boundary on its left, `.l`, and on its right, `.r`. Prefix sums live on those boundaries. There is no question of whether to allocate `n` or `n + 1`, or whether to subtract at `l - 1`.

## One idea, fewer special cases

A vertex, an edge, and a character position are all bounded indices. Arrays, grids, and strings use the same indexing mechanism. A row or a slice is temporary access to existing storage, not a hidden copy.

The language core stays small. Domains expand to sealed structs implementing `Index`. Containers and their projections are library code. You can understand the rules once, then use them in your own library.

## A path through the language

1. [Getting started](getting-started.html) — install the development CLI and run a prototype example.
2. [A quick tour](tour.html) — values, optional results, loops, functions, and program structure.
3. [Index spaces & bounds](indices.html) — put positions and the gaps between them in different types.
4. [Ownership & borrowing](borrowing.html) — use `lends`, not lifetime annotations.
5. [ASCII strings](strings.html) — flat character arrays with their own index spaces.
6. [Memoization & stress tests](testing.html) — cache recurrences and compare fast code with a brute.
7. [The standard library](stdlib.html) — reuse the familiar tools, or write your own.

## Know what you can run

> **Development edition.** This guide describes the approved language design, not a finished release. Blocks marked **prototype tested** compile and run in the Python prototype. Blocks marked **design** explain the intended language and are not advertised as runnable. The Rust CLI currently checks a subset and serves the LSP; its run/build backends are not built yet.

The [getting-started chapter](getting-started.html#what-works-today) separates working commands from the target workflow. There is no pretend Run button: examples run locally against the actual prototype. The [design decisions](https://github.com/ryanbai1412/cp-lang/blob/main/decisions.md) and [plan](https://github.com/ryanbai1412/cp-lang/tree/main/plan) explain the rationale and remaining work.
