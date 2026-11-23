# Writing conformance cases

| | |
|---|---|
| Kind | Guidelines |
| Owner | spec |
| Describes | `spec/conformance/`, `spec/SUBSET.md`, `crates/cpl-conform/`, `scripts/conformance-*.py`, `scripts/gen-programs.py` |

Read this before you add a case, mark a known gap, or build a suite over the
spec. A case is a short program plus the one result the spec promises for
it. The file format is in `spec/conformance/README.md`. This doc says what
makes a case worth having. Every check that enforces an item cites it by
anchor.

## A good case, by example

```cp-fragment
// expect: ok, output "20 10"
// rule: 07.4.7 07.4.11
// A piece of a `.rev` is cut in walk order, not from the source's positions.
let a = [10, 20, 30]
print(a.rev.drop(1))
```

It tests one rule, names it, and has an expected line you can check by
hand from the spec text alone. The source and its mirror image give
different answers (`20 10` against `30 20`), so a mirrored implementation
fails. A case where both readings print the same thing tests nothing.

## How to think about a case

A case is worth having when it can fail. Work out the answer from the spec
before running anything, and pick inputs where a plausible wrong reading
gives a different answer. Then ask where the bug class lives as a space
(every index type, every consumer, every trait method) and generate the
table instead of writing one example by hand. A case listed as a known gap
still has to be right: if it fails for a reason other than the one it
names, it hides the bug it was written for.

Every case also runs in both engines. The interpreter and the generated C++
must agree, under sanitizers too, and random programs whose output we can
compute independently catch the wrong answers both engines share.

The full list, with ids that checks cite, is [conformance-checklist.md](conformance-checklist.md).

## Changes

- 2026-11-23: first version; rules moved to the checklist.
