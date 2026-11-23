# Changing the compiler

| | |
|---|---|
| Kind | Guidelines |
| Owner | compiler |
| Describes | `crates/cpl-syntax/`, `crates/cpl-check/`, `crates/cpl-ir/`, `crates/cpl-interp/`, `crates/cpl-prover/`, `spec/conformance/`, `crates/cpl-cli/tests/programs/` |

Read this before you change how a program is parsed, typed, lowered or run.
The parts and their invariants are in `handbook/design/compiler.md`; this
page says how to change them without breaking a user.

## A good fix, by example

A user reports that `fn max(a: Int, b: Int) -> Int = a + b` is ignored by
`max(x, 3)`: the builtin runs and prints the wrong number.

A bad fix adds `if name == "max" && self.declares("max")` in the call
lowering. It fixes one name, leaves `min` and `abs` broken, and adds a std
name to the compiler.

A good fix finds the rule that was broken (a local name shadows a prelude
name, R11.2.3), finds the one place names are resolved, and makes every
prelude name obey it there. It lands with a conformance case for `max`, and
the same case with a top-level `let max = ...`, and records where each name
resolves so lint and the editor read the same answer.

## The rules

1. **Fix the class, not the case.** Find the rule in `spec/` the program
   broke and the one place that rule lives, and fix it there. Never match a
   std name as a string, add a one-case path, or add an edition branch: each
   is counted by `ir_builtin_names`, `prelude_privilege` or
   `backend_std_names` in `scripts/debt-ratchet.py` and may only go down.
2. **Every fix lands with its regression.** The conformance case or
   program that exposed the bug is committed with the fix. When the two
   engines could differ, add a program under `crates/cpl-cli/tests/programs/`
   so `crates/cpl-cli/tests/differential.rs` holds both to one `.out`.
3. **Never weaken a case.** A conformance case that contradicts `spec/` is a
   question for spec, not an edit. A skip, xfail or allowlist entry only
   ever shrinks.
4. **One program, two engines.** The VM and the C++ run the same IR and must
   agree on every valid program and on input either rejects. A fact both
   need (effects, pushed loops, printing) is decided once in `cpl-ir` and
   read by both; a back end never re-derives a type the checker gave.
5. **A bound means what it says.** A type meets a trait bound only by
   implementing the trait. A view such as `Rev<S>` is never unwrapped to `S`
   to satisfy a bound, and a generic is bound only to what its uses need: a
   generic the fn uses as a domain may bind to a range, any other keeps the
   range's `Int`.
6. **No error starts inside std.** Generic std is checked at its
   definition, so a user's mistake is reported at the user's call.
7. **Check what users already wrote.** A change to typing or lowering is
   tried on every corpus port, acceptance solution and site reference
   before it lands; a change that breaks one is migrated with it, in the
   same commit, keeping each program's meaning.

## Checklist

- [ ] The fix sits where the broken spec rule lives, with no std name,
      one-case path or edition branch added.
- [ ] A conformance case or differential program fails without the fix
      and passes with it.
- [ ] No conformance case, expected output or skip list was loosened.
- [ ] `cargo nextest run -p cpl-check -p cpl-ir -p cpl-interp` and
      `cargo nextest run -p cpl-cli --test differential` pass.
- [ ] `cpl-conform all --check` shows no regression, which covers the
      corpus ports and the spec blocks.
- [ ] `python3 scripts/debt-ratchet.py --check` passes.

## Measures

The compiler charter (`handbook/domains/compiler.md`) lists every quality
the compiler must have and the suite that enforces it: conformance, the
two-engine differential, VM fallbacks, std privilege and std checked at
definition.

## Changes

- 2026-11-23: first version, from the compiler lane's working rules and
  the review of fixes that broke corpus programs.
