# std docs checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | stdlib |
| Describes | every doc comment in `std/*.cp`; the reasoning is in [std-docs.md](std-docs.md) |

Every rule a std doc comment must meet, one per heading. Each has a stable
id (`std-docs/R3`), one sentence, its reason, an example and what enforces
it. Tests, scripts and the [stdlib quality register](../domains/stdlib.md)
cite these ids instead of restating the rule.

# Reference

## Content

### R1. Every public fn has `cost:`, `traps:` and `example:`

Each `pub fn` doc carries the three required labels.

- **Why:** cost, failure and a copyable call are what a contestant needs mid-contest.
- **Example:** `cost: \(O(\log n)\)`, `traps: never`, ``example: `[3, 1].sorted() == [1, 3]` ``.
- **Enforced by:** `scripts/spec-stub.py --audit`, counted in `plan/domains/stdlib/metrics.toml`.

### R2. `empty:`, `ties:` and `order:` wherever they apply

A fn that can see empty input says what it returns; one that picks among equals says which; one that yields many values says in what order.

- **Why:** these are the edge cases contestants get wrong.
- **Example:** `empty: None`, `ties: the first`, `order: increasing key`.
- **Enforced by:** review; `missing` (generated edge-case suite driven by the labels).

### R3. The cost names its variables and every phase

`cost:` uses `\(O(...)\)` in terms of named inputs and gives build and query costs separately.

- **Why:** "fast" can't be compared with a time limit; \(O(n \log n)\) can.
- **Example:** ``cost: \(O(n)\) to build, \(O(1)\) per `fold` ``.
- **Enforced by:** review; `missing` (scaling test per fn, time at n and 4n).

### R4. `traps:` lists every trap, or says `never`

Every condition that traps is listed in words, in source terms.

A `trap:` line under it is a one-line program that must trap, so the doc
cannot claim a check the code skips.

- **Why:** a trap the doc didn't mention reads like a compiler bug, and a
  documented trap that doesn't fire returns a wrong answer silently.
- **Example:** ``traps: when `c` is not in `0..cols` `` with
  ``trap: `Mat<Int>.zeros(2, 2).get(0, 2)` ``.
- **Enforced by:** `scripts/std-trap-examples.py` runs every `trap:` line;
  `--audit` lists the `traps:` lines with no `trap:` yet.

### R5. The rendered doc shows trait bounds and defaults

`cpl doc`, hover and the site show each generic's bounds and each option's default.

- **Why:** a contestant needs to know a type must be `Ord` before calling, not from the error.
- **Example:** hover on `sorted` shows `T: Ord` and `by: T -> K = x => x`.
- **Enforced by:** `missing` (doc JSON shape test over `cpl doc --format json std`).

## Examples

### R6. Every example compiles and runs

Each `example:` line is a `Bool` expression that is `true`, run on both engines.

- **Why:** contestants copy examples; a stale one teaches a wrong call.
- **Example:** ``example: `[3, 1, 4].scan(+).fold(1..=2) == 5` ``.
- **Enforced by:** `cpl-conform docs`, ratcheted in `cpl-conform all --check`; `missing` as a hard gate per example.

### R7. The first example is the common case

The first example shows what most callers do, with the shortest input that shows it; option and edge examples follow.

- **Why:** a reader stops at the first example.
- **Example:** for `scan`, `[3, 1, 4].scan(+).prefix(2) == 4` before the `from:` example.
- **Enforced by:** review.

## Readability

### R8. One plain summary sentence first

The doc starts with one sentence saying what the item is for, without restating the signature.

- **Why:** hover shows the first line; it must stand alone.
- **Example:** "The prefix folds of `self` under `f`, so any range fold is O(1)."
- **Enforced by:** `missing` (doc readability lint: summary length, one sentence).

### R9. The spec's words, and links instead of repetition

Docs use the spelling of `spec/GLOSSARY.md` and link the spec section for long explanations.

- **Why:** one vocabulary across spec, errors, trace and docs.
- **Example:** "cut", not "boundary index"; "domain", not "index type".
- **Enforced by:** `scripts/spec-vocab.py`.

### R10. Each module opens with what it is for

A `//!` line names the module's job and its main types.

- **Why:** it is the first thing go-to-definition and the site's module page show.
- **Example:** ``//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`. ``
- **Enforced by:** review; `missing` (module-doc presence check).

## Verification

### R11. A page counts as verified only after a checklist walk

A module's docs are verified when someone has checked every public item against R1 to R10 and recorded the date in `plan/domains/stdlib/docs-tracker.md`.

- **Why:** label counts say a doc exists, not that it is right.
- **Example:** the tracker row for `prefix.cp` lists the date and the rules that still fail.
- **Enforced by:** process; the tracker's count of verified modules only goes up.

## Changes

- 2026-11-23: first version.
