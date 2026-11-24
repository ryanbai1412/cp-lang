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
- **Enforced by:** `scripts/spec-gate.py std-docs --audit`, at 0 in `scripts/check.sh`.

### R2. `empty:`, `ties:` and `order:` wherever they apply

A fn that can see empty input says what it returns; one that picks among equals says which; one that yields many values says in what order.

- **Why:** these are the edge cases contestants get wrong.
- **Example:** ``empty: `SortedBag<Int>().first == None` ``, ``ties: `[3, 1, 1].argmin() == 1` ``, `order: increasing key`. An `empty:` or `ties:` label is a claim (`a == b`, `a != b`, `not a`, or code tagged `(traps R0401)`), so it runs as a case.
- **Enforced by:** `scripts/std-docs-lint.py` fails a public fn over a `[D]T` with no `empty:`; `cpl-conform docs` runs every claim label in the interpreter and in the C++ build (`crates/cpl-conform/src/docs.rs` `runnable_label`, `native_labels`); the `edge_labels_unrun` ratchet counts the labels that are still prose or a bare value.

### R3. The cost names its variables and every phase

`cost:` uses `\(O(...)\)` in terms of named inputs and gives build and query costs separately.

- **Why:** "fast" can't be compared with a time limit; \(O(n \log n)\) can.
- **Example:** ``cost: \(O(n)\) to build, \(O(1)\) per `fold` ``.
- **Enforced by:** `bench/scaling.py --check` times each fn at n and 4n against its `cost:` (in `scripts/check.sh`).

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
- **Enforced by:** `scripts/test_std_doc_json.py` over `cpl doc --format json std`.

## Examples

### R6. Every example compiles and runs

Each `example:` line is a `Bool` expression that is `true`, run on both engines.

- **Why:** contestants copy examples; a stale one teaches a wrong call.
- **Example:** ``example: `[3, 1, 4].scan(+).fold(1..=2) == 5` ``.
- **Enforced by:** `cpl-conform docs`, ratcheted in `cpl-conform all --check`; per example, `web/scripts/gen-ref.mjs` fails the site build on one that stops running (`site/R1`; the known failures are the shrink-only `web/scripts/doc-examples-failing.txt`).

### R7. The first example is the common case

The first example shows what most callers do, with the shortest input that shows it; option and edge examples follow.

- **Why:** a reader stops at the first example.
- **Example:** for `scan`, `[3, 1, 4].scan(+).prefix(2) == 4` before the `from:` example.
- **Enforced by:** `scripts/std-docs-lint.py` fails a first example that passes a defaulted option while a later one passes none.

## Readability

### R8. One plain summary sentence first

The doc starts with one sentence saying what the item is for, without restating the signature.

- **Why:** hover shows the first line; it must stand alone.
- **Example:** "The prefix folds of `self` under `f`, so any range fold is O(1)."
- **Enforced by:** `scripts/std-docs-lint.py` (a summary paragraph exists and is at most 240 characters; shrink-only, in `scripts/check.sh`).

### R9. The spec's words, and links instead of repetition

Docs use the spelling of `spec/GLOSSARY.md` and link the spec section for long explanations.

- **Why:** one vocabulary across spec, errors, trace and docs.
- **Example:** "cut", not "boundary index"; "domain", not "index type".
- **Enforced by:** `scripts/spec-gate.py vocab`, in `scripts/check.sh`.

### R10. Each module opens with what it is for

A `//!` line names the module's job and its main types.

- **Why:** it is the first thing go-to-definition and the site's module page show.
- **Example:** ``//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`. ``
- **Enforced by:** `scripts/std-docs-lint.py` fails a std module whose first line is not `//!`.

## Changes

- 2026-11-23: first version.
- 2026-11-23: R11's manual walk retired: R1 to R10 each run in `scripts/check.sh`.
