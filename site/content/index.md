Start with the [interactive course](course.html) for six progressive chapters with worked examples and checked exercises, keep the [one-page cheat sheet](cheat-sheet.html) open during contests, or run code directly in the [offline playground](playground.html).

CP Lang is a language for competitive programming that compiles to clean, readable C++17. Keep the algorithms you know. Replace index arithmetic, sentinel values, and repeated bookkeeping with types and helpers that say what you mean.

Start with [your first program](getting-started.html), take the [language tour](tour.html), or go straight to [index spaces](indices.html).

```cp Prefix sums · CP Lang 0.2.2
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

A vertex, an edge, a grid cell `(Row, Col)`, and an enum variant all share one bounded-index mechanism (`Index`). When you just want a plain integer-indexed array without declaring a domain, write `[0..n]T` or `[0..=k]T` and reset it with `a.fill(x)`. Strings (`Str`) are flat, value-semantic `0..len`-indexed ASCII byte sequences, with `[domain S]Char` available when you want nominal character positions.

The language core stays small: 34 reserved keywords (30 for contestants, 4 for library authors), 4 numeric types, 7 auto-derived core traits, and zero lifetime annotations. Missing values are `?T` rather than sentinels, and `?Int` behaves as a tropical number: `+` absorbs `none`, `min=` skips it. Containers, `lends` projections, `Monoid(id, op)`, `Action`, `compress(all)`, and `Rooted` are ordinary userland code you can read and extend in your own local library.

## CP Lang 0.2.2 Showcase

Five real contest submissions from `corpus/submissions/` rethought idiomatically in CP Lang (`proto/ports/`). Both raw C++17 line counts and library-stripped counts (removing template headers and standard data structures) are reported from each batch's verified `loc.py`.

Across the whole corpus, the [registry](https://github.com/ryanbai1412/cp-lang/blob/main/bench/corpus-500-report.md) counts 573 unique problems ported. 477 of them have a port that a harness differentially checked against the original submission; those 477 ports are 66.4% shorter than the raw originals and 48.4% shorter than the library-stripped originals. Over all 573 the figures are 66.1% and 47.9%.

```showcase
```

## A path through the language

1. [One-page cheat sheet](cheat-sheet.html) — every CP Lang 0.2.2 construct, collection, and stdlib helper on one page.
2. [Getting started](getting-started.html) — install `cpl` and run your first native binary.
3. [A quick tour](tour.html) — values, `?T` optionals, parallel assignment, explicit undo, and `@multitest`.
4. [Index spaces & bounds](indices.html) — `[0..n]T`, nominal domains, `.l`/`.r` boundaries, `p.adj4`/`p.adj8`, and `compress(all)`.
5. [Ownership & borrowing](borrowing.html) — explicit `mut`, `lends`, and `@law disjoint` without lifetime annotations.
6. [ASCII strings](strings.html) — value-semantic `0..len`-indexed `Str` and opt-in `[domain S]Char`.
7. [Memoization & stress tests](testing.html) — `@memo`, `@brute`, `@checker`, `@interactor`, and shrinking.
8. [The standard library](stdlib.html) — `bsearch_first`, the collections, bare ops / `Monoid(id, op)`, `Action`, `desc(x)`, and `Rooted`.

## Know what you can run

> **Development edition.** This guide teaches the locked **CP Lang 0.2.2** specification (`docs/one-page-cheat-sheet.md`, `decisions.md` §16–§19). Blocks marked **prototype tested** compile and run in the Python prototype and in the offline WebAssembly [playground](playground.html). Blocks marked **CP Lang 0.2.2** show the locked language design where the Rust compiler and prototype are still catching up.

The [getting-started chapter](getting-started.html#what-works-today) separates working commands from planned ones. The [design decisions](https://github.com/ryanbai1412/cp-lang/blob/main/decisions.md) and [0.2.2 architecture](https://github.com/ryanbai1412/cp-lang/blob/main/research/language-architecture-v2.2.md) record the rationale for every feature.
