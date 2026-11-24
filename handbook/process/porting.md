# Porting accepted solutions

| | |
|---|---|
| Kind | Process |
| Owner | idioms |
| Describes | `corpus/problems/<id>/`, `research/frictions/`, `research/reviews/`, `scripts/corpus.py`, `scripts/review-check.py` |

A port is an accepted C++ solution rewritten in CP Lang and kept in the
corpus beside the original. It is evidence about the language: each port
shows what reads well, what is missing and what is slow. So the bar is the
exemplars, not "it passes the tests".

## Example

Codeforces 2147F lives in `corpus/problems/cf2147f/`:

```text
corpus/problems/cf2147f/
  orig.cpp    the accepted C++ submission
  dream.cp    the program as it should read in ideal CP Lang, written first
  port.cp     the program as close to the dream as today's language allows
  gen.py      a generator; tests.toml lists its seeds (older problems: tests/)
  meta.json   source, artifacts and tags
```

Its port opens with the standard header, then the program:

```cp
// Source: Codeforces 2147F (Exchange Queries), https://codeforces.com/contest/2147/problem/F
// Problem: two permutations p and s rank n <= 1e5 items; item i trades for item j when
//   p[i] > p[j] or s[i] > s[j]. After each of q <= 1e5 swaps, print how many pairs
//   (i, j) have j reachable from i. Multitest.
// Solution: cut c is free when no item has one value on each side; free cuts split the
//   items into SCCs. A SegTree with range add keeps the cuts at the minimum cover and
//   their gap sum; a swap re-adds the two changed items, O(log n) per query.

@!multitest
let n: 2..=100_000 = read()
let q: 1..=100_000 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]Cut = read()
var s: [Item]Cut = read()
```

The bounds from the statement are types, the two index spaces are domains,
and the header says the idea before the code does. Every gap between this
port and its dream is in `research/frictions/cf2147f.md`, and a cold
reviewer's score is in `research/reviews/cf2147f.md`.

The gold standard is `research/design-history/EXEMPLARS-PORTS.md`: five
real ports written as they should read. `research/design-history/EXEMPLARS.md`
and `research/design-history/sketches/` show more of the style.

## The steps

1. **Understand the problem, not the C++.** Read the statement and the
   editorial idea. Check the original is complete and passes the samples.
2. **Write the dream first**, in `corpus/problems/<id>/dream.cp`, before
   you look at what compiles: the program as it should read, inventing
   syntax and library calls freely in the exemplars' style. It is not
   gated or formatted. If the dream reads like the C++, start over.
3. **Write the port** as close to the dream as the language allows. Gate
   it with `cpl check` and `cpl test corpus/problems/<id>/port.cp` (or
   `just conform corpus <id>`), and run it against `orig.cpp` on generated
   inputs. Measure tokens, runtime on a max input against the C++, and
   emitted bytes.
4. **Report every gap** between port and dream in
   `research/frictions/<id>.md`: the snippet, the dream lines that remove
   it, the goal it hurts, a general fix (never a one-problem helper), and
   its severity. A gap that says the language lacks something needs a run:
   try the dream line with `cpl run` on a freshly built `cpl` and quote the
   error. Route each gap to the domain that owns it.
5. **Self-review against the taste checklist**, then commit the port,
   dream, friction report and `research/reviews/<id>.md` holding
   `Score: pending` together, as `port(<id>): <what the program does>`.

Ports never edit `std/`, `crates/` or `spec/`. A missing library function or
a compiler bug is a friction for its owner, with a repro, not a change made
from inside a port.

### Size and speed

A rewrite toward the dream may grow the emitted C++ only when its review
names the runtime part that grew and the codegen ticket that removes it.
Accept the growth by rebaselining in the same commit
(`just conform --write`, which updates `research/scoreboard.json`). The
48 KB warn line still blocks. A port slower than its C++ by more than 1.1x
on a max input needs a performance friction.

## The header

Every `port.cp` opens with three comment blocks, then `// original:` if
there is one:

```cp
// Source: Codeforces 2147F, https://codeforces.com/contest/2147/problem/F
// Problem: <the statement in 1-4 lines: input, what to compute, the bounds that matter;
//   enough to solve it without opening the statement>
// Solution: <a 2-6 line sketch: the key observation, then the algorithm and its cost,
//   enough to picture the solution and follow the code>
```

`uv run python scripts/corpus.py verify --only header` names every port
missing one of the three; `scripts/check.sh` blocks on any.

## Taste checklist

A port fails review if it breaks a rule in
`handbook/guidelines/idiomatic-code-checklist.md` and the gap is not
reported in its friction file. Self-review against that checklist.

A dream fails review, and caps at 3, if it breaks one of those rules: it
keeps the C++ construction where the editorial names an object or rule
(`idiomatic-code/R2`), hides a side effect (`idiomatic-code/R7`),
writes a fact twice (`idiomatic-code/R4`), or mixes spellings
(`idiomatic-code/R8`).

## Review

Every new port gets a cold taste review: a fresh agent with only
`AGENTS.md`, this file, the exemplars, the statement, the dream and the
port, never the C++. It scores the port 1 to 5 against the dream and lists
the failures; a 5 reads like an exemplar. It also scores the dream: a dream
that reads like the C++ is sent back before the port is judged. A port with
no dream scores at most 3. Ports under 4 go back to their author with the
review. Recurring findings become domain work or checklist lines.

The review lives in `research/reviews/<id>.md` (format and score scale in
`research/reviews/README.md`), and its score shows in `research/PORTS.md`.

## What enforces this

| Rule | Check |
|---|---|
| Every new port has a dream and a review | `scripts/review-check.py`, in `scripts/check.sh --fast` |
| New ports under 4 only go down | The ratchet in `research/reviews/baseline.json`, read by `scripts/review-check.py`; lower it when a rewrite lifts a port, never raise it |
| Ports compile and lint clean | The commit tier runs `cpl check` and the corpus lint ratchet on each staged `port.cp` and `spec.cp` |
| Ports agree with their tests | `cpl-conform corpus` and `cpl test` |
| Ports carry the header | `scripts/corpus.py verify` (`header`; blocks in `scripts/check.sh`) |
| Dream agrees with the accepted C++ on the package's tests (or gen.py seeds) | `scripts/corpus.py verify --check`, on every staged dream in the commit tier |
| Port is close to its dream | `scripts/review-check.py` reports the port-over-dream token gap; `scripts/corpus.py gaps` counts what the dreams need from the language (neither blocks) |

## Changes

- 2026-11-23: first version, absorbing `plan/PORTING.md`.
- 2026-11-23: the taste rules moved to `handbook/guidelines/idiomatic-code.md`.
- 2026-11-23: rules cited by checklist id; the example layout is a tree.
