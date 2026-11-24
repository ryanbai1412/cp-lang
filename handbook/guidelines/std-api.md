# Designing std APIs

| | |
|---|---|
| Kind | Guidelines |
| Owner | stdlib |
| Describes | every public signature in `std/*.cp` and `std/prelude.cp`; the exact rules are in [std-api-checklist.md](std-api-checklist.md) |

Read this before you add or rename anything public in the standard library
(std). The goal is simple to state: a contestant who knows one std type
should guess the calls of the next one without opening its docs, and the
call they guess should be correct and as fast as it looks.

## A good API, by example

```cp
let c = [3, 1, 3].tally()     // Counter: c[3] is 2, c[7] is 0
let best = scores.max() ?? 0  // ?T: an empty list has no max
rows.sort(by: r => (-r.score, r.name))
let sums = a.scan(+)  // sums.fold(l..=r) in O(1)
```

Each line uses a name the reader has met in another language or another
std module, says "nothing" with `?T` instead of a magic value, passes
options by name, and costs what it looks like it costs.

A bad one: `lower_bound(a, x)` returning `a.len` when nothing is found. The
name is C++'s, the "not found" answer is a valid-looking `Int`, and the
caller who forgets to compare it with `len` reads past the end.

## The ideas behind every rule

**One word per idea, everywhere.** `len`, `is_empty`, `add`, `remove`,
`push`, `pop`, `first`, `last` mean the same thing on every type. A new
idea gets a new word once, in the lexicon, and every later module reuses
it.

**"Nothing" is a type, not a value.** Anything that can fail to find,
or be asked of an empty input, returns `?T`. A sentinel like `-1` or `INF`
type-checks and then silently corrupts the answer.

**Options are named, with defaults.** `sort(by: key)`, `bfs(g, from: s)`.
One function grows an option instead of the library growing a `_by`
twin.

**Indices keep their index space.** Over a `[D]T`, positions are `D`,
cuts between them are `Cut<D>`, and ranges are `Range<D>`. A bare `Int`
loses the fact that lets the checker catch an off-by-one.

**One form per operation.** A fold of one value is a method (`xs.sum()`),
math on several values is a free function (`gcd(a, b)`). Having both by
accident means two things to learn and two to maintain.

**Costs are visible.** Anything that looks O(1) is O(1), and a copy looks
like a copy. A getter never hides work.

**Operations carry their identity.** `+`, `min`, `max`, `gcd` and other
monoids know their identity, so a fold of them returns `T`. An operation
with no identity still works, and its fold returns `?T`.

**Nothing common is missing.** Every common operation from the C++ STL,
Python's builtins and `itertools`, Rust's iterators and collections, and
the AtCoder Library has a CP Lang spelling, or is a listed gap.

**Each type's everyday tasks are one call each.** For a sorted container
that means insert, erase one, erase at a cursor, predecessor and
successor. A task that needs a loop or a sentinel check is an API bug.

**No privilege.** std uses only powers a user library has; see
[std-mechanisms.md](../design/std-mechanisms.md).

## Deciding a case nobody wrote down

Ask what a contestant would type first, having used the neighbouring std
types and one mainstream language. If that spelling is unambiguous, use
it. If two spellings are equally likely, pick the one already used
elsewhere in std. If the obvious spelling would be subtly wrong or slow,
change the design until it isn't. Then add the rule to the checklist, so
the next person doesn't decide it again.

## How we check it

[std-api-checklist.md](std-api-checklist.md) lists every rule with an id
(`std-api/R3`), its reason, an example and what enforces it. Today the
vocabulary ban list (`scripts/spec-gate.py vocab`), the spec-signature check
(`scripts/conformance-sigs.py`) and the trait-coverage check enforce some
of it. The weakest areas are the API-consistency lint over std signatures
and the usage-journey suite, which don't exist yet; reviews catch the rest.

## Where to start

- **Adding a function:** find its words in the lexicon (`std-api/R1`), then
  check the rest of the checklist before landing.
- **Adding a type:** write its journeys first (`std-api/R13`), one tested
  call each.
- **Renaming:** ship the codemod and a checker error naming the new
  spelling (`std-api/R20`).

## Changes

- 2026-11-23: first version, from the naming rules in `std/README.md`.
- 2026-11-23: overview first; rules moved to the checklist.
