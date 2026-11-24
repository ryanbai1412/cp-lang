# # Writing conformance cases: checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | spec |
| Describes | `spec/conformance/`, `spec/SUBSET.md`, `crates/cpl-conform/`, `scripts/conformance-*.py`, `scripts/gen-programs.py` |

Every rule for a conformance case, numbered so checks and reviews can cite it as `conformance/R<n>`. The reasons and the picture are in [conformance.md](conformance.md).

## Rules

### conformance/R1. Every rule has a case, and every forbidding rule has a reject case

A rule with no case can drift without anyone noticing. A rule that forbids
something needs a program the checker must reject, with the code it must
give, not only programs that follow the rule. Enforced by
`scripts/conformance-coverage.py --check` and `spec/conformance/rule-gaps.txt`,
which may only shrink.

### conformance/R2. Work out the expected result from the spec, not from the compiler

Write the expected output before you run the program. Take it from the
rule's text and the std signatures: `xs.max()` is a `?T`, so the case writes
`xs.max()!`. A case copied from today's output pins today's bugs. Pick inputs
whose wrong readings differ, as in the example above: asymmetric values,
both sides of a boundary, empty and one-element inputs.

### conformance/R3. A known gap fails for the reason it names

A case with a `// gap: <owner> <YYYY-MM-DDTHHZ>` line (decision 515) is a
known gap and is expected to fail, so a
mistake in the case itself (a wrong `// <- here` line, a stale API) hides until the
implementation catches up. Before you open a gap, check that the failure is
the bug the case is about. Enforced by `scripts/conformance-gaps.py`
(the case's own code at another line, or a printed `?T` without `!`) and by
`scripts/spec-gate.py gaps` (every failing case has a gap line, and a gap
that starts passing must be dropped) and `gap-age` (no gap past 48 h, none
re-dated).

### conformance/R4. Every fixed bug lands with a case that fails without the fix

A fix without a case can regress silently. The case has to fail when the
fix is reverted. `scripts/conformance-bugcatch.py` reverts recent fixes and
counts how many some case catches.

### conformance/R5. Where a rule covers a space, generate the cases

Index types times consumers, every trait method on every type that has it,
every std signature a chapter names, every type a prelude signature names:
generate these cases from a table, don't write them by hand. A table shows
the empty cells. Enforced by `scripts/conformance-matrix.py`,
`conformance-traits.py`, `conformance-sigs.py` and `conformance-prelude.py`,
each run with `--check`.

### conformance/R6. Results agree across engines, with and without sanitizers

Every `ok` case runs in the interpreter and as built C++, and the two must
agree. Random programs whose output a model computes catch wrong answers
the two engines share. Enforced by `cpl-conform diff`, by the same run with
`CPL_CONFORM_CXXFLAGS='-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG'`,
and by `scripts/gen-programs.py`.

### conformance/R7. Code in the spec runs

Every `cp` block in `spec/` is checked as a program. A block that is
deliberately partial says so with `<!-- spec-only: why -->`, or with
`<!-- given: ... -->` for the lines it assumes. Enforced by
`cpl-conform spec-blocks --check`.

### conformance/R8. The suite stays fast enough to run on every land

Every land runs the conformance and spec-block suites, so their cost is
paid many times a day. Their total CPU time has a budget of about twice
today's; a case or harness change that doubles it is a regression to fix,
not a new baseline. CPU time is the measure because wall time moves with
machine load. Enforced by `scripts/conformance-speed.py`.

### conformance/R9. The suite catches bugs no past fix shows

The bug-catch rate replays bugs that once happened; a suite tuned to it can
still miss the next one. So small bugs are planted in the compiler (a
comparison flipped, `&&` for `||`, `+ 1` for `- 1`) and the suite rerun.
A surviving mutant either needs a case or is shown to change nothing a
program can see, and the report says which. Example: reusing the wrong
map's entry under two nested `if k in m:` tests survived until a case read
two maps with the same key. Enforced by `scripts/conformance-mutants.py`,
in the non-blocking `ci-slow` workflow, with its report in
`plan/domains/spec/mutants.md`.

## Checklist

- [ ] One rule per case, cited in `// rule:` (conformance/R1)
- [ ] Expected result taken from the spec, and wrong readings give a different one (conformance/R2)
- [ ] A known gap's failure is the bug it names; the gap names an owner (conformance/R3)
- [ ] The fix's case fails with the fix reverted (conformance/R4)
- [ ] Cases over a space come from a table (conformance/R5)
- [ ] Runs the same in both engines and under sanitizers (conformance/R6)
- [ ] Keeps the suites within their CPU budget (conformance/R8)
- [ ] A surviving mutant gets a case or is shown equivalent (conformance/R9)

## Changes

- 2026-11-23: first version.
