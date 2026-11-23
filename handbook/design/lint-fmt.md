# Lint and fmt design

| | |
|---|---|
| Kind | Design |
| Owner | lint-fmt |
| Status | Active |
| Describes | `crates/cpl-lint`, `crates/cpl-fmt`, `spec/lints/` |

The linter is a set of small rules that read what the checker already knows
about a file and say where legal code is probably wrong, slow or not the
CP Lang way. The formatter re-emits the file's tokens with one canonical
layout and proves it changed nothing else. Neither has its own idea of what
a program means: the checker owns that.

Example: the editor holds a checker database for `main.cp`. On each edit
it asks the database for the file's analysis, passes it to
`cpl_lint::lint_analysis`, and shows the findings. `cpl lint main.cp`
builds the same analysis once and makes the same call, so the terminal and
the editor never disagree. `cpl fmt main.cp` lexes the file, lays the
tokens out, re-lexes the result and refuses to write it unless the tokens
match.

## What good looks like

- **One analysis.** Every tool lints the analysis its checker database
  already holds; no tool builds a database just to lint.
- **Facts, not guesses.** A rule reads types, resolved names and ranges from
  the checker. A rule that needs a fact the checker lacks asks for it there.
- **A rule is data plus a matcher.** Each lint's metadata, documentation
  and tests are one record under `spec/lints/<name>/`; the Rust side is
  only the matcher.
- **The formatter can't change meaning.** Its only freedom is whitespace and
  trailing commas, and it checks that after every run.

## How it fits together

```
source text
  -> cpl_check database: parse, names, types, facts, checker diagnostics
  -> cpl_lint::lint_analysis(analysis, config)
       rules/   each rule walks the items and reads checker facts
       levels   cp.toml [lints], groups, `with lint(name: level)` regions
  -> findings, each with an optional fix
  -> cpl_lint::fix_in / apply_fixes: apply non-overlapping fixes to a
     fixed point, keep the result only if it checks

source text
  -> cpl_syntax::lex -> cpl_fmt layout (spacing, indentation, breaking)
  -> sort `use` runs -> verify: re-lex, same tokens, or refuse
```

## Why some rules exist

Two lints exist because the corpus wrote one idea several ways, and the
language picked one. The lint and its `cpl fix` codemod are how old code
reaches the chosen spelling.

**One name per tree traversal (decision 63).** Ports spelled preorder five
ways: `t.Pre`, `t.preorder`, `t.postorder`, `tree.post_order` and
`t.preorder.rev`. Now `for p in t.Pre` loops over positions, and vertex loops
read the stored preorder array: `for v in t.at` visits parents first and
`for v in t.at.rev` children first. Each is built from general mechanisms (an
array is iterable, `.rev` is a view), so nothing new needs learning, and the
loop reads `at` in place where `preorder` built a list. `tree_order` (L0511)
rewrites `.preorder` to `.at` and `.postorder` to `.at.rev`. Alternatives that
lost:
- Keep `preorder` and `postorder` as getters. That gives two names for one
  array, and each call copies it.
- Make `t.Pre` yield vertices. That would need a special iteration rule in the
  core for one type.
- A true postorder with siblings in order. Contest code rarely needs one; it
  stays `t.at.rev` until a port shows otherwise.

**Filters are `if`, power is `**` (decision 70).** Comprehension filters
appeared as `for x in xs if p` 543 times and as `where p` 38 times. Power
appeared as `a ** b`, `a.pow(b)` and once as `a ^ b`, though `^` is xor. Now
filters are `if`, which most code already used and which matches Python.
Power is `**`, and `^` means only xor. `cpl fix` rewrites `where` filters to
`if` and `.pow(e)` to `** e`. Alternatives that lost:
- Keep `where` as a parser alias. That leaves two spellings for one idea.
- Keep `.pow` as a method. That leaves a second form of `**`.
- Use `^` for power. That clashes with xor, and C++ habit reads it as xor.

## Reference

### Modules

| Where | What it is for |
|---|---|
| `crates/cpl-lint/build.rs` | Generates `LINTS`, one `Lint` per record in code order, from each `lint.toml`, and checks each record's `rule` exists in its checklist |
| `crates/cpl-lint/src/lib.rs` | `Level`, `Lint`, the generated registry; `lint_analysis`, `lint`, `fix_in`, `apply_fixes`; `with lint(...)` regions; the removed-syntax codemods behind `cpl fix` |
| `crates/cpl-lint/src/checker.rs` | Sorts what the checker reported into checker lints at their levels |
| `crates/cpl-lint/src/config.rs` | `Config` from `cp.toml [lints]` |
| `crates/cpl-lint/src/walk.rs` | Tree walkers the rules share |
| `crates/cpl-lint/src/rules/` | The rules, one family per file; `rules/taste/` holds the idiom families, `rules/proven/` the range analyses the editor's hints read |
| `crates/cpl-fmt/src/lib.rs` | `format`, `format_with`, `is_formatted`, `verify` |
| `crates/cpl-fmt/src/{spacing,layout,imports,range}.rs` | Spacing between tokens; indentation, comments and line breaking; sorting `use` runs; range and on-type formatting |

### Invariants

- Fmt changes only whitespace and trailing commas (`verify` in
  `crates/cpl-fmt/src/lib.rs`), and is idempotent.
- Fmt formats files that don't parse; it needs only the lexer.
- Lint never blocks `cpl build`. A file that doesn't check is never
  reported lint-clean.
- A lint's code and name never change meaning and are never reused.
- `apply_fixes` never applies two overlapping edits, and a fix is kept only
  if the fixed file checks.

### What it must never do

- Re-derive a type, a resolved name or a range from syntax
  (`handbook/guidelines/lints-checklist.md`, `lints/R4` and `lints/R5`).
- Name a std item by spelling instead of by what it resolves to
  (ratchet `lint_std_names` in `scripts/debt-ratchet.py`).
- Build a checker database of its own inside a tool
  (ratchet `own_databases`).
- Reorder tokens in fmt without a verify step for that move.

## Changes

- 2026-11-23: first version, folded from `plan/domains/lint-fmt/ARCHITECTURE.md`.
