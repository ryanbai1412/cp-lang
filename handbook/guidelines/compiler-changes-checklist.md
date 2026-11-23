# Changing the compiler: checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | compiler |
| Explained by | [compiler-changes.md](compiler-changes.md) |

Every rule a compiler change must meet, each with a stable id. Tests,
ratchets and reviews cite the id; the compiler's quality register in
`handbook/domains/compiler.md` points here instead of restating them.

## Rules

### compiler-changes/R1: fix the rule where it lives

A fix goes where the broken spec rule is implemented, never as a match on a
std name, a one-shape path or an edition branch.

- Reason: a name-matched fix covers one case and leaves the class broken.
- Example: user `fn max` shadowing is fixed in name resolution, for every
  prelude name, not with `if name == "max"`.
- Enforced by: `ir_builtin_names`, `prelude_privilege`, `backend_std_names`
  and `lint_std_names` in `scripts/debt-ratchet.py`, which only shrink.

### compiler-changes/R2: every fix lands with its regression

The conformance case or program that exposed the bug is committed with the
fix, and fails without it.

- Reason: a bug without a test comes back.
- Example: a `scan(+)` fix adds `crates/cpl-cli/tests/programs/scan_range_elem.cp`
  with its `.in` and `.out`.
- Enforced by: review at landing.

### compiler-changes/R3: never weaken a case

No conformance case, expected output, skip, xfail or allowlist entry is
loosened to land a change. A case that contradicts the spec goes to spec.

- Reason: a weakened case hides the regression it was written to catch.
- Example: a case expecting `30 20` stays; the fix makes the compiler
  print it.
- Enforced by: `cpl-conform all --check`; skip-list counts in
  `scripts/debt-ratchet.py`.

### compiler-changes/R4: both engines run one program

The interpreter and the C++ run the same lowered `Program`, and a fact both
need is computed once in `cpl-ir` and read by both.

- Reason: two derivations of one fact drift apart into wrong answers.
- Example: which globals a closure writes comes from `cpl_ir::effects`,
  not from a second analysis in codegen.
- Enforced by: `cpl-conform diff`; `crates/cpl-cli/tests/differential.rs`
  over `crates/cpl-cli/tests/programs/`.

### compiler-changes/R5: a bound is met only by implementing it

A type satisfies a trait bound only if it implements the trait; a view such
as `Rev<S>` is never unwrapped to `S` to pass a bound.

- Reason: unwrapping silently changes meaning, here the walk order.
- Example: `prefix_matches(t.rev, p)` is an error because `Rev<Str>` is not
  `Indexed`; `prefix_matches(List(t.rev), p)` works.
- Enforced by: conformance cases in `spec/conformance/core-07/`.

### compiler-changes/R6: a generic binds only what its uses need

A generic is inferred from how the function uses it: one used as a domain
may bind to a range, any other keeps the range's element type.

- Reason: over-binding turns a working call into a confusing error inside
  std.
- Example: in `scan<D, T>` over `[I]-9..=9`, `D` is `I` and `T` is `Int`.
- Enforced by: `crates/cpl-cli/tests/programs/scan_range_elem.cp` and
  `range_elem_binds_domain.cp`.

### compiler-changes/R7: a local name shadows a prelude name

A name the file declares wins over a prelude name of the same spelling,
whatever its arity (R11.2.3).

- Reason: ignoring the user's own function is a silent wrong answer.
- Example: with `fn max(a: Int, b: Int) -> Int = a + b`, `max(4, 3)` is 7.
- Enforced by: conformance cases in `spec/conformance/core-11/`.

### compiler-changes/R8: no error starts inside std

Generic std is checked at its definition, so a mistake in a user program is
reported at the user's code.

- Reason: an error pointing into `std/` gives the user nothing to edit.
- Example: a bad call to `scan` points at the call, not at `std/prefix.cp`.
- Enforced by: `crates/cpl-cli/tests/std_checks.rs`.

### compiler-changes/R9: the front end never panics

Parsing and checking finish on any input, with at most one error per root
cause.

- Reason: a crash in the editor loses the user's whole feedback loop.
- Example: a half-typed `for x in` still gives a tree and one error.
- Enforced by: `crates/cpl-cli/tests/recovery.rs`.

### compiler-changes/R10: the VM runs everything itself

The VM never hands a construct to the old tree-walker.

- Reason: two interpreters are two meanings.
- Example: `swap(mut a[i], mut a[j])` on a packed array runs on the VM.
- Enforced by: the `vm_fallbacks` ratchet and `CPL_VM_STATS`.

### compiler-changes/R11: migrate what users already wrote

A change to typing or lowering is tried on every corpus port, acceptance
solution and site reference before it lands; one it breaks is migrated in
the same commit, keeping its meaning.

- Reason: a working program that stops building is a regression for the
  user who wrote it.
- Example: when `a.min()` became `?Int`, ports comparing it with `<` moved
  to `a.min()!` in the same change.
- Enforced by: `cpl-conform all --check` (corpus and spec blocks) and the
  site gate in CI.

### compiler-changes/R12: a new syntax form reaches every tool's suite

A statement or expression form that any suite's programs use also appears
in the programs of every other tool's suite: C++ differential, LSP, fmt,
trace and lint.

- Reason: a feature isn't done until it works in every tool, and a suite
  can only show that for forms its programs contain.
- Example: `unchecked:` blocks appear in conformance cases, so a saved-trace
  program in `crates/cpl-cli/tests/programs/` uses one too.
- Enforced by: `every_syntax_form_reaches_every_tools_suite` in
  `crates/cpl-cli/tests/tool_matrix.rs`; today's gaps are listed in
  `crates/cpl-cli/tests/tool-gaps.txt`, which `skip_lists` only lets shrink.

## Before you land

- [ ] R1: no std name, one-shape path or edition branch added.
- [ ] R2: a case or program fails without the fix and passes with it.
- [ ] R3: nothing loosened.
- [ ] R4: `cargo nextest run -p cpl-cli --test differential` passes.
- [ ] R11: `cpl-conform all --check` shows no regression.
- [ ] `cargo nextest run -p cpl-check -p cpl-ir -p cpl-interp` and
      `python3 scripts/debt-ratchet.py --check` pass.

## Changes

- 2026-11-23: first version.
