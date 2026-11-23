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
reason and how to reverse it. `scripts/next-decision.py --check` keeps the
numbering unique.

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

## Checklist

- [ ] The most general rule, with the special cases it replaces removed (spec-rules/R1)
- [ ] Decision logged with why and how to reverse it (spec-rules/R1)
- [ ] One spelling; options are named arguments; alternates get a fix (spec-rules/R2)
- [ ] Rule id at the end of the sentence; references say `see` (spec-rules/R3)
- [ ] Meaning, proof, and per-build check all stated (spec-rules/R4)
- [ ] Matching `std` signature and prelude export (spec-rules/R5)
- [ ] `handbook/language/` updated if the model changed (spec-rules/R6)
- [ ] A case for the rule (`guidelines/conformance.md`)

## Changes

- 2026-11-23: first version.
