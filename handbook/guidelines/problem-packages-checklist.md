# Problem package checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | idioms |
| Explains | `handbook/guidelines/problem-packages.md` |

Every rule a corpus problem folder or acceptance judge must follow, each
with a stable id that audits and tests cite. The ideas behind them are in
`handbook/guidelines/problem-packages.md`.

# Reference

### problem-packages/R1

**Rule:** every expected output is what an accepted submission prints on that input.
**Why:** a hand-written or copied answer passes wrong solutions and fails right ones.
**Example:** `python3 scripts/corpus-tests.py make <id>` writes `tests/NN.ans` from the accepted C++.
**Enforced by:** `scripts/corpus-tests.py verify`; `corpus/audit.py` (`no-reference`: tests but no accepted C++ with a `main`).

### problem-packages/R2

**Rule:** no expected output equals its input (beyond two tokens).
**Why:** it means the answer was copied, not computed.
**Example:** a `tests/09.ans` byte-identical to `tests/09.in`.
**Enforced by:** `corpus/audit.py` (`answer-is-input`).

### problem-packages/R3

**Rule:** a problem with more than one valid output has a `checker.py`, and is judged by it.
**Why:** token comparison fails correct answers that differ from the reference.
**Example:** `python3 checker.py <in> <out> <ans>`, exit 0 accepts.
**Enforced by:** `cpl-conform corpus` runs it; `corpus/audit.py` (`needs-checker`) flags two accepted submissions that disagree on a test.

### problem-packages/R4

**Rule:** an interactive problem has an interactor and a generator for its hidden inputs.
**Why:** without them it is judged on nothing, or not like the judge.
**Example:** `corpus/problems/cco18p4/interactor.py` and `gen.py`.
**Enforced by:** `corpus/audit.py` (`interactor-no-gen`).

### problem-packages/R5

**Rule:** generated inputs satisfy every constraint in the statement.
**Why:** an invalid input tests a case the judge never gives.
**Example:** a validator that rejects `n = 0` when the statement says `1 <= n`.
**Enforced by:** `corpus/generators.py` (`gen-fails`, `invalid-input`, `invalid-test`): the port's `read` types are the validator.

### problem-packages/R6

**Rule:** the package has an input at the statement's bounds, shaped to be slowest for the obvious solution, judged on the C++ build.
**Why:** the time limit is exercised, not assumed.
**Example:** `max.py` in an acceptance judge.
**Enforced by:** `research/acceptance/judge.py`; corpus: `corpus/generators.py` (`no-max`, and the `--max` input must pass the port's C++).

### problem-packages/R7

**Rule:** tests run under the interpreter finish within its per-case limit (20 s); larger inputs belong to the C++ run.
**Why:** a slow interpreter case times out on loaded CI runners and turns main red for everyone.
**Example:** a 2.7 MB input moved from `tests/` to the max-input run.
**Enforced by:** `cpl-conform corpus` (per-case limit).

### problem-packages/R8

**Rule:** the corpus, acceptance judges, `cpl test` and `cpl stress` read one layout: `N.in` with `N.out` or `N.ans`, `checker.py`, `interactor.py`, `gen.py`.
**Why:** one format means one set of tools and no conversions that drift.
**Example:** `corpus/problems/cf2147f/`.
**Enforced by:** `corpus/differential.py` runs `cpl test` on every v03 package and reports the cases it can't find or judge (`cpl-test-*`).

### problem-packages/R9

**Rule:** no literal test data is committed under `corpus/`: a problem's tests are its `gen.py` plus a seed list in `tests.toml`, with only tiny hand-picked inputs inline there.
**Why:** committed inputs can't be regenerated or resized and bloat the repository forever; a seed list is reviewable and a size change is a one-line edit.
**Example:** `[[case]] seed = 7 size = 200000` in `tests.toml`; `scripts/corpus-tests.py make` writes inputs and answers to `target/corpus-tests/<id>/`.
**Enforced by:** `scripts/corpus-tests.py make` writes only under `target/`; `scripts/debt-ratchet.py` (`corpus_test_files`, shrink-only) and `scripts/check.sh --commit`, which refuses a newly added test file. `corpus/audit.py` (`stale-tests-path`) fails when a tracked file outside `corpus/` still names a seeded problem's deleted `tests/`.
