# Problem packages

| | |
|---|---|
| Kind | Guidelines |
| Owner | idioms |
| Describes | `corpus/problems/<id>/`, `research/acceptance/problems/`, `scripts/corpus-tests.py`, `research/acceptance/judge.py` |

A problem package is everything needed to judge a solution offline: the
statement's limits, a recipe for the tests, a trusted way to get each
answer, and the programs that check and interact. Read this before you add
a corpus problem or an acceptance judge. A wrong package is worse than none:
it passes wrong solutions or fails right ones, and nobody looks again. The
exhaustive rules, each with an id, are in
`handbook/guidelines/problem-packages-checklist.md`.

## What good looks like

```text
corpus/problems/cf2147f/
  meta.json     source, limits, accepted submissions
  orig.cpp      the accepted C++ the answers come from
  gen.py        gen.py SEED SIZE prints one input
  tests.toml    the cases: a seed and size each, or a tiny input inline
  checker.py    only if several outputs are valid
  interactor.py only if the problem is interactive
```

`tests.toml` is a recipe, not data:

```toml
[[case]]
name = "n1"
input = "1\n7\n"

[[case]]
name = "max"
seed = 7
size = 200000
```

`python3 scripts/corpus-tests.py make cf2147f` turns it into inputs and
answers under `target/corpus-tests/cf2147f/`, running `gen.py` for each
seed and the accepted C++ for each answer. Every reader (`cpl test`,
`cpl-conform corpus`, the perf bench, the stress tests) reads that one
cache.

Older problems still commit `tests/NN.in` and `NN.ans`; they move to
`tests.toml` one by one, and no new test files are committed.

## The ideas behind the rules

**Tests are generated, never committed.** A committed input can't be
regenerated, reviewed or resized, and large ones bloat the repository
forever. A seed list is a few lines, and changing a size is a one-line
edit.

**Answers are computed, never written.** Every expected output is what an
accepted solution prints. A problem with several valid outputs is judged by
a checker, and an interactive one by talking to an interactor, as on the
judge.

**Inputs are real.** Generated inputs satisfy the statement, and at least
one is at the statement's bounds, so the time limit is exercised rather
than assumed. Interpreter runs stay small; the biggest inputs belong to the
compiled C++.

**One format for every tool.** The corpus, acceptance judges, `cpl test`
and `cpl stress` share the layout above, so a package built once works
everywhere.

## Changes

- 2026-11-23: first version, from the corpus and acceptance judge practice.
- 2026-11-23: tests are `gen.py` and `tests.toml`, with answers in one cache
  under `target/`; rules moved to `problem-packages-checklist.md`.
