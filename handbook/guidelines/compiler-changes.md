# Changing the compiler

| | |
|---|---|
| Kind | Guideline |
| Owner | compiler |
| Describes | `crates/cpl-syntax/`, `crates/cpl-check/`, `crates/cpl-ir/`, `crates/cpl-interp/`, `crates/cpl-prover/` |

Read this before you change how a program is parsed, typed, lowered or
run. It explains how to fix a compiler bug so the fix holds for every
program, not only the one in the report. The exhaustive rules, each with an
id that tests and reviews cite, are in
[compiler-changes-checklist.md](compiler-changes-checklist.md).

## A good fix, by example

A user writes their own `fn max(a: Int, b: Int) -> Int = a + b` and calls
`max(x, 3)`. The builtin runs instead and prints the wrong number, with no
error.

A bad fix adds a check for the name `max` in the call lowering. It fixes
one name, leaves `min` and `abs` broken, and teaches the compiler one more
std name.

A good fix starts from the spec. The rule broken is "a local name shadows a
prelude name" (R11.2.3). Names are resolved in one place, so the fix goes
there and every prelude name obeys it at once. The commit adds a
conformance case for a user `fn max` and one for a top-level
`let max = ...`, so the bug can't come back unseen.

## The ideas behind the rules

**The spec decides, the compiler obeys.** Every bug is a spec rule the
compiler broke. Find the rule, find the one place it is implemented, and
fix it there. A fix that matches a std name, adds a path for one shape of
program, or special-cases an edition is a symptom patch; it is counted as
debt and only allowed to shrink.

**One program, two engines.** The checked interpreter and the generated
C++ run the same lowered program. A fact both need, such as which globals a
function writes or how a type prints, is decided once and read by both.
Then a disagreement between them is always a bug in one place, and the
differential suite finds it.

**Types mean what they say.** A trait bound is met only by a type that
implements the trait, and a generic is bound only to what its uses need.
Guessing a type to make a program compile turns a clear error into a wrong
answer later.

**What users already wrote keeps working.** Corpus ports, acceptance
solutions and site examples are real programs. A change that breaks one
either migrates it in the same commit, keeping its meaning, or is the
wrong change.

**Evidence lands with the fix.** The program that exposed a bug becomes a
test in the same commit, and no existing test is loosened to make room.

## Deciding a case nobody wrote down

Ask which spec rule the program depends on. If none covers it, the
question goes to spec before any code changes. If one does, ask where that
rule lives in the compiler, and whether your fix makes every program the
rule covers right, not only this one.

## Changes

- 2026-11-23: first version; rules moved to the checklist.
