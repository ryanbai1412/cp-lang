# Documenting std

| | |
|---|---|
| Kind | Guidelines |
| Owner | stdlib |
| Describes | the `///` and `//!` doc comments in `std/*.cp`, and what `cpl doc`, hover and the site show from them; the exact rules are in [std-docs-checklist.md](std-docs-checklist.md) |

A std doc comment is the only documentation most contestants read, usually
as an editor hover in the middle of a contest. It has a few seconds to
answer three questions: what does this do, what does it cost, and what
happens at the edges. Read this before you write or change one.

## A good doc, by example

```cp
/// The prefix folds of `self` under `f`, so any range fold is O(1).
///
/// cost: \(O(n)\) to build, \(O(1)\) per `fold`
/// empty: one cut, holding `from` or the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never (beyond what `f` traps on)
/// example: `[3, 1, 4].scan(+).fold(1..=2) == 5`
pub fn scan<D, T, M>(self: [D]T, f: M, from: ?T = None) -> Prefix<D, T>:
```

The first line says what it is for in plain words. The labelled lines
answer the questions a contestant actually has: the cost, the empty case,
the key order, the traps. The example is one line, shows the common case,
and runs: the doc test suite executes it on every change, so it can't go
stale.

A bad one: "Computes prefix sums using an efficient algorithm." It names
no cost, says nothing about empty input or traps, and has no example to
copy.

## The ideas behind every rule

**Examples first, and every example runs.** A contestant copies the
example before reading the prose. An example that doesn't compile is
worse than none.

**Labels answer fixed questions.** `cost:`, `traps:` and `example:` are on
every public fn; `empty:`, `ties:` and `order:` wherever they apply. A
reader learns where to look once, and tools can check that the answer is
there.

**Say the bounds.** The signature's trait bounds (`T: Ord`,
`M: Monoid<T>`) are part of what the fn needs, so the rendered doc shows
them, not just the names.

**Short and plain.** One summary sentence, then labels. Use the spec's
words for the spec's ideas, and define nothing that the glossary already
defines.

**Verified, not hoped.** A page is "done" when someone has checked it
against the checklist and recorded that in the tracker.

## How we check it

[std-docs-checklist.md](std-docs-checklist.md) lists every rule with an id
(`std-docs/R4`). `scripts/spec-gate.py std-docs --audit` finds fns missing a
required label, and `cpl-conform docs` runs every `example:` line. Every rule
R1 to R10 runs in `scripts/check.sh`. The weakest areas today are that a
failing example is ratcheted rather than a hard gate and that rendered docs
don't yet show trait bounds.

## Where to start

- **Writing a new fn's doc:** copy the shape above, then run
  `target/release/cpl-conform docs <module>.cp`.
- **Verifying a page:** walk the checklist over one module and record the
  result in the tracker.

## Changes

- 2026-11-23: first version.
