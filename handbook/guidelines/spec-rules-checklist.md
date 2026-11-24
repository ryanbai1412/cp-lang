# # Writing spec rules: checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | spec |
| Describes | `spec/core/`, `spec/lib/`, `spec/tools/`, `decisions.md`, `handbook/language/` |

Every rule for a spec rule, numbered so checks and reviews can cite it as `spec-rules/R<n>`. The reasons and the picture are in [spec-rules.md](spec-rules.md).

## Rules

### spec-rules/R1. Prefer one general rule, and prefer removing to adding

A friction reported for one case usually has a general cause. Fix the
cause with the rule that covers the most cases, and remove the special
cases it replaces. A new name or form with one use means the general
mechanism is missing. Each such call goes in `decisions.md` with its
reason and how to reverse it. Take its number with
`scripts/next-decision.py --reserve`; `scripts/land.sh` refuses a number
main never had.

### spec-rules/R2. One spelling per idea, picked by type rather than by name

Each operation has one name. The argument's type picks the behavior, and
options are named arguments: the real-valued binary search is
`(0.0..8.0).bsearch(ok, steps: 60)` on a `Range<F64>`, not a separate
`bsearch_real`. An alternate spelling either gets a diagnostic with a
machine fix (`else if` gives E0503, fixed to `elif`) or doesn't exist.
`scripts/spec-vocab.py` checks the vocabulary across the spec.

### spec-rules/R3. Every rule has an id, every reference says `see`

A rule is defined where `(Rnn.n.n)` ends its sentence. A mention elsewhere
is written `(see Rnn.n.n)`, so tools don't read it as a second definition.
Enforced by `scripts/conformance-coverage.py --check`, which also flags
rules no case cites (conformance C1).

### spec-rules/R4. Every checked operation answers three questions

What it means, what the compiler can prove, and what each build checks.
If one of the three is missing, an implementer has to guess, and two
implementers will guess differently.

### spec-rules/R5. Chapters and `std` say the same thing

A signature, a return type or a named option written in a chapter must
match `std/*.cp`, and every type a prelude signature names is in the
prelude (R11.3.2). Enforced by `scripts/conformance-sigs.py --check` and
`conformance-prelude.py --check`, with `sig-gaps.txt` and
`prelude-gaps.txt` only shrinking.

### spec-rules/R6. The handbook and the spec change together

A rule change that alters an area's model or a key decision updates that
area's `handbook/language/` doc in the same commit.

### spec-rules/R7. A removed spelling is in the changelog

Every spelling the spec makes E0503 (removed syntax) is named, with its
replacement, under the current version's breaking changes in
`CHANGELOG.md`, because an old solution meets it first as an error. The
release notes are generated from that file. Example: `D.bounds` is
`D.cuts` appears as a breaking-changes line. Enforced by
`scripts/spec-changelog.py`.

### spec-rules/R8. Every tool feature has a rule id

A tool feature in `spec/tools/` (a trace column, a debugger command, an
explain section) is a promise like a language rule, so it gets an
`(Rnn.n.n)` id and a case that cites it. Without an id no case can name
it, and the coverage gate can't tell it is untested. Example: "`debug(x)`
prints with a source location in checked builds and is removed from
submissions" needs an id, a run case, and a build case that `lacks:` it.
Enforced by `scripts/spec-tool-rules.py`, whose count only goes down.

## Checklist

- [ ] The most general rule, with the special cases it replaces removed (spec-rules/R1)
- [ ] Decision logged with why and how to reverse it (spec-rules/R1)
- [ ] One spelling; options are named arguments; alternates get a fix (spec-rules/R2)
- [ ] Rule id at the end of the sentence; references say `see` (spec-rules/R3)
- [ ] Meaning, proof, and per-build check all stated (spec-rules/R4)
- [ ] Matching `std` signature and prelude export (spec-rules/R5)
- [ ] `handbook/language/` updated if the model changed (spec-rules/R6)
- [ ] A case for the rule (`guidelines/conformance.md`)
- [ ] Every removed spelling has a changelog line (spec-rules/R7)
- [ ] A new tool feature has a rule id and a case (spec-rules/R8)

## Changes

- 2026-11-23: first version.
