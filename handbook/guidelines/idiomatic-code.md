# Writing idiomatic CP Lang

| | |
|---|---|
| Kind | Guidelines |
| Owner | idioms |
| Describes | `corpus/problems/*/port.cp`, `dream.cp`, `spec.cp`, `skills/cp-lang/`, `spec/idioms/`, examples in docs and on the site |

Read this before you write a port, a dream, a skill page or a docs example.
Every program we publish is read as the way CP Lang is meant to be written,
by people and by models, so each one has to be the version we would teach.

## By example

A translated loop from the C++:

```cp
var best = -1
var at = -1
for i in 0..n:
    if a[i] > best:
        best = a[i]
        at = i
print(at + 1)
```

The idiomatic version:

```cp
domain I = 1..=n
let a: [I]1..=1e9 = read()
print(I.max(by: i => a[i]))
```

The statement's bound is a type, the index space is a domain, so there is
no `+ 1` to forget, and the search is a named library call with no `-1`
sentinel.

## The rules

### Read like the editorial

The invariant, recurrence or algorithm is visible, not buried in index
bookkeeping. The header (`Source:`, `Problem:`, `Solution:`) says the idea
before the code does, and derives every constant and closing formula the
code uses. Keep the C++ construction only when the editorial uses it too.

### Use the standard structure or algorithm

Don't hand-roll what std or the language provides: a DFS for a tree path or
diameter, a search loop with a `?T` flag where `find`, `first` or `any`
belongs, a run-length loop where `runs()` belongs. If std lacks it or is
measured too slow, keep the hand-rolled version and report the gap with a
run that shows it.

### Each fact in one place

A bound, base or modulus is written once. Nothing is kept in sync by hand:
no symmetric matrices filled twice, no parallel arrays, no sentinel
bookkeeping. A loop or formula is never written twice.

### Constraints are types

Every bound from the statement that today's types can hold is in a type:
`let x: 1..=10_000_000 = read()`, not `let x: Int = read()`. Absent values
are `?T`, never `-1` or another sentinel.

### Index spaces are domains

When two index spaces differ (vertices and edges, positions and cuts, raw
and compressed), each is a `domain`, and values never cross through a raw
`Int`.

### No hidden effects

An expression doesn't hide a side effect (`first(xs, x => try_add(x))`). No
result is discarded with `let _ =`. State a closure could own isn't
threaded by hand.

### One spelling per idea

Use the spec's spelling and only that: `@!multitest`, `Some(x)` and `None`,
`Mod<P>`, `read<T>`, `struct`, `.items` for index and element, and methods
for sequence operations on one sequence (`xs.sum()`, `xs.any(p)`,
`xs.sorted()`). The free form stays canonical over a generator
(`sum(f(x) for x in xs)`) and over two or more values (`min(a, b)`).

### No new warnings

`cpl check` and `cpl lint` pass, and the corpus warning counts don't go up.
A lint you believe is wrong goes to its owner, not into an `allow`.

### As fast as the C++

On a max input a port runs within 1.1x of its matched C++. If it doesn't,
file a performance friction with the measurement
(`handbook/guidelines/performance.md`).

### A gap is a claim that needs a run

A friction saying the language lacks something includes the dream line run
under a freshly built `cpl` and the error it gives.

## Checklist

- [ ] The header has `Source:`, `Problem:` and `Solution:`.
- [ ] No hand-rolled standard algorithm without a reported gap.
- [ ] Every bound and index space from the statement is a type or domain.
- [ ] No sentinel, duplicated fact or hidden effect.
- [ ] Only the spec's spellings.
- [ ] `cpl check` and `cpl lint` are clean.
- [ ] Within 1.1x of the C++ on a max input, or a friction is filed.

## What enforces this

| Rule | Check |
|---|---|
| Use the standard structure or algorithm | the hand-rolled-std lint family; `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`) |
| No new warnings | the commit tier's `cpl check` and lint ratchet on staged ports |
| Read like the editorial; each fact in one place | cold taste review (`handbook/process/porting.md#review`), ratcheted by `scripts/review-check.py` |
| The header | `corpus/header.py` |
| One spelling per idea | the spec's removed-spelling errors in `cpl check`; a scan of the skill and docs: `missing` |
| Constraints are types | `missing` (bounds in `meta.json` and a max-bound run) |
| As fast as the C++ | perf's corpus bench |

## Changes

- 2026-11-23: first version, from the taste checklist in `handbook/process/porting.md` and `research/reviews/README.md`.
