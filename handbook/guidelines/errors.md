# Writing errors, warnings and traps

| | |
|---|---|
| Kind | Guidelines |
| Owner | diagnostics |
| Describes | `crates/cpl-diag/`, `spec/diagnostics.md`, error sites in `crates/cpl-syntax/`, `crates/cpl-check/`, `crates/cpl-ir/`, trap messages in `crates/cpl-interp/`, `crates/cpl-cli/tests/diagnostics/` |

Read this before you add an error, change a message or add a fix. It applies
to every error (`E`), warning (`W`), lint (`L`) and runtime trap (`R`).

## A good error, by example

```
error[E0107]: `&&` is `and` in CP Lang
  --> main.cp:3:6
  |
3 | if a && b:
  |      ^^
  = help: replace with `and`
```

It points at the two characters to change, says the rule in a few words,
and carries a fix that `cpl fix` and the editor apply.

A bad one, for `rows.sort(by: (a, b) => (-b, a))` where `rows` holds
records:

```
error[E0309]: `-` can't be applied to a `(name: Str, score: Int)`
```

The real mistake is that `by:` takes a key of one element, not a
comparator of two. The error blames `-`, which the user wrote correctly for
what they meant, and gives no fix. It should point at `(a, b)`, say that
`by:` takes one element and returns its key, and offer
`rows.sort(by: r => (-r.score, r.name))`.

## The rules

1. **One error per root cause.** Report the first thing the user got wrong,
   and nothing that follows from it. Push through `Sink`, which drops a
   second diagnostic inside a span already reported. If your check runs
   after another error, give a poisoned (`Ty::Error`) value no further
   error.
2. **Point at the place to edit.** The primary span is in the user's file,
   on the token they must change. Never put it inside `std/`: an error that
   arises in a library names the user's call as the primary span and the
   library line in a note. Use a label for the other half: the `let` of a
   written name, the first of two `mut` borrows.
3. **Say the rule in the user's words.** Name the construct the user wrote
   (`` `&&` ``, `` `by:` ``), not the parser's state (`expected an
   expression`). Use the spec's vocabulary from `spec/GLOSSARY.md`: one
   spelling per idea, the same in the error, the docs and the trace. Cite
   the rule id `(R05.2.1)` when the rule is not obvious.
4. **Give the fix.** If one edit is right, attach it with `Diagnostic::fix`
   so `cpl fix` and the editor can apply it. If several are plausible,
   attach each; the fix applier keeps only those after which the program
   checks clean. If no edit is right, say what to do in `help`, and never
   offer a placeholder that compiles but is wrong.
5. **Only suggest things that exist.** A did-you-mean, a rename or an import
   must name a public item that runs. Free functions are suggested as
   free calls (`zip(a, b)`), methods as methods (`a.max()!`).
6. **Show types as the user wrote them.** Print aliases, domains and views
   (`Rev<List<Int>>`) by their names, not by internal representations.
7. **A trap names values.** The line, the values that broke the rule, the
   call stack, the test case number in multitest, and the CP Lang way to
   avoid it.
8. **A warning must be true.** A lint warns only when it can show the
   problem: a bound lint names an input inside the declared ranges that
   overflows. If the analysis can't see why a value is safe, stay silent.

## Codes

Every diagnostic has a code from `spec/diagnostics.md`, used through a
`codes::` constant. A code never changes meaning, and a retired code is
never reused. A new code needs, in the same commit:

- a row in `spec/diagnostics.md`, with an example message;
- a `REGISTRY` entry in `crates/cpl-diag/src/registry.rs`, with a trigger
  that emits exactly that code and, if the error has one, a fix that makes
  the trigger check clean;
- an explanation shown by `cpl explain <code>`;
- for a lint, a record in `spec/lints/<name>/` or `spec/checker-lints/<name>/`:
  `lint.toml`, `explain.md`, a `before.cp` that fires it, and `after.cp` and
  `near_miss.cp` that don't.

## Checklist

- [ ] The primary span is on the token to edit, in the user's file.
- [ ] The message names what the user wrote and the rule, in the spec's words.
- [ ] A right fix is attached, or `help` says what to do.
- [ ] Every name in the message and the fix exists and is public.
- [ ] The error does not repeat for later uses of the same mistake.
- [ ] A golden in `crates/cpl-cli/tests/diagnostics/` pins the text, and
      `cargo test -p cpl-cli --test diagnostics --test catalogue` passes.
- [ ] `cargo test -p cpl-lsp parity` passes, so the editor shows the same.

## Measures

The diagnostics charter (`handbook/domains/diagnostics.md`) lists every
quality an error must have and the suite that enforces it. The main ones:
first error on the broken line, cascades, silent broken programs and fix
coverage (`crates/cpl-cli/tests/recovery.rs`), newcomer habits
(`crates/cpl-cli/tests/newcomer.rs`), and fixes keeping program output
(`crates/cpl-cli/tests/fix_behavior.rs`).

## Changes

- 2026-11-23: first version, from the diagnostics judgment rubric and
  catalogue rules.
