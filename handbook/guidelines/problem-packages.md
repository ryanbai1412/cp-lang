# Problem packages

| | |
|---|---|
| Kind | Guidelines |
| Owner | idioms |
| Describes | `corpus/problems/<id>/`, `research/acceptance/problems/`, `scripts/corpus-tests.py`, `research/acceptance/judge.py` |

A problem package is everything needed to judge a solution offline: the
statement's limits, tests with trusted answers, and the programs that
generate, check and interact. Read this before you add a corpus problem or
an acceptance judge. A wrong package is worse than none: it passes wrong
solutions or fails right ones, and nobody looks again.

## By example

`corpus/problems/cf2147f/`:

| File | Rule it satisfies |
|---|---|
| `orig.cpp` | Answers come from an accepted solution |
| `tests/sample1.in`, `sample1.out` | Samples first |
| `gen.py` | Generated cases, including a max input |
| `meta.json` | Source and submissions |

A problem with several valid answers adds `checker.py`
(`python3 checker.py <in> <out> <ans>`, exit 0 accepts). An interactive one
adds `interactor.py`.

## The rules

### Answers come from an accepted solution

Every expected output is what an accepted submission prints on that input,
reproducible with `python3 scripts/corpus-tests.py verify`. Never write an
answer by hand or copy it from the program under test. An expected output
equal to its input is always a bug.

### Several valid answers need a checker

If a problem accepts more than one output, it has a checker, and the
package is judged by it, never by token comparison. Two accepted
submissions that print different outputs on one test prove the problem
needs a checker.

### Interactive problems need an interactor

An interactive problem ships an interactor and a generator for its hidden
inputs, and is judged by talking to it, as on the judge.

### Inputs are valid

Generated inputs satisfy every constraint in the statement. A generator
that produces an invalid input tests a solution against a case the judge
never gives.

### The max input is a test

Every package has at least one input at the statement's bounds, shaped to
be slowest for the obvious solution, so the time limit is exercised, not
assumed.

### One format

The corpus, acceptance judges, `cpl test` and `cpl stress` read the same
layout: `N.in` with `N.out` or `N.ans`, `checker.py`, `interactor.py`,
`gen.py`. A tool that needs another layout changes the format here first.

## Checklist

- [ ] Samples, generated cases and a max input.
- [ ] Every answer reproduced by an accepted solution.
- [ ] A checker if several answers are valid; an interactor if interactive.
- [ ] Generated inputs within the statement's constraints.

## What enforces this

| Rule | Check |
|---|---|
| Answers come from an accepted solution | `scripts/corpus-tests.py verify`; no answer equal to its input: `missing` |
| Several valid answers need a checker | `cpl-conform corpus` runs `checker.py`; detector from disagreeing accepted submissions: `missing` |
| Interactive problems need an interactor | `missing` as a coverage check |
| Inputs are valid | `missing` (a validator per problem) |
| The max input is a test | `research/acceptance/judge.py` for acceptance; corpus: `missing` |

## Changes

- 2026-11-23: first version, from the corpus and acceptance judge practice.
