# Writing lints

| | |
|---|---|
| Kind | Guidelines |
| Owner | lint-fmt |
| Describes | `crates/cpl-lint/`, `spec/lints/`, `scripts/new-lint.py`, `cp.toml [lints]`, `with lint(...)` |

Read this before you add, change or retire a lint. A lint flags legal code
that is probably wrong, slow or not the CP Lang way. It never blocks a
build. The bar is a strong contestant who reads the warning and thanks it.
Checker lints (`L06xx`) and bound lints over declared ranges (`L03xx`)
follow `handbook/guidelines/errors.md` as well.

## A good lint, by example

`spec/lints/push_loop/before.cp`:

```
let raw = [3, 1, 2]
var doubled: List<Int> = []
for x in raw: doubled.push(2 * x)
print(doubled)
```

`cpl lint` says:

```
warning[L0507]: a loop that only pushes onto `doubled` is a comprehension
  --> spec/lints/push_loop/before.cp:2:1
  = help: write `[2 * x for x in raw]`; the comprehension says what each item is, so no push is missed or doubled
```

The message names the fact. The help gives the exact replacement and the
reason in one line. `cpl fix` rewrites it to the record's `after.cp`:

```
let raw = [3, 1, 2]
let doubled = [2 * x for x in raw]
print(doubled)
```

The record's `near_miss.cp` holds the shapes that look alike but must stay
quiet, such as a loop that pushes only some items and reads the list
while it pushes.

A bad lint matches `.len` or `max` as text. It then fires on a user's own
`fn max`, or on a type that happens to have a `len` field. It guesses that
`d` is a float because its name is `d`, and repeats an error the checker
already reports at the same place.

## The rules

1. **Precision first.** A lint that cries wolf teaches users to ignore all
   lints. If a rule can't be sure, it stays silent or warns without a fix.
   Idiomatic code (corpus `spec.cp`, `std/`, the practice solutions) gets no
   warnings. A finding there is a bug in the lint, not in the code.
2. **Read the checker's facts, never guess them.** Types, resolved names,
   ranges and effects come from the checker's analysis (`cpl_lint::Facts`),
   the same one the editor holds. Don't infer a type from a declaration,
   a name or a literal. If a fact you need is missing, ask the compiler
   domain for it rather than re-deriving it.
3. **Match what a name resolves to, not how it is spelled.** A rule about
   `std.seq`'s `sum` checks the name's resolved module, so a file's own
   `fn sum` is never flagged. Lint rules name no std API by spelling; the
   `lint_std_names` ratchet only goes down.
4. **Don't restate the checker.** If a checker error already covers the
   shape, put the fix on that error (diagnostics owns it) and retire the
   lint. A lint never fires at the span of a checker diagnostic.
5. **Every fix is safe to apply without reading it.** A fix must clear its
   own lint, still parse, add no checker error, keep CRLF line ends and keep
   what the program prints. `cpl fix` applies a fix only after the result
   re-checks clean. When the right edit depends on intent, give help text
   instead of a fix, and say why in the record.
6. **Teach in one read.** The message says what the code does in the user's
   words. The help gives the exact CP Lang spelling, using the spec's
   vocabulary (`spec/GLOSSARY.md`) and only names that exist. Every lint has
   an `explain.md` page that `cpl explain L0507` prints.
7. **One finding per cause.** Two lints never teach one shape. Nested
   findings are either one cause, reported once, or listed as distinct.
8. **Easy to turn off.** Every lint has a name and a group (`correctness`,
   `perf`, `idiom`, `style`, `pedantic`). One line turns either off:
   `cp.toml [lints]` (`push_loop = "allow"`, `style = "allow"`) or
   `with lint(push_loop: allow)` around the code.
9. **Cheap and linear.** A rule is one pass over the tree and the facts.
   Lint runs on every keystroke in the editor.

## Adding a lint

1. `python3 scripts/new-lint.py <name> --group idiom --level warn` takes
   the next free code and creates `spec/lints/<name>/` with `lint.toml`,
   `before.cp`, `after.cp`, `near_miss.cp` and `explain.md`. Land the record
   soon after, since two lanes can take the same code.
2. Write `before.cp` (fires once), `after.cp` (what the fix writes; it must
   pass `cpl check`) and `near_miss.cp` (every similar shape that must stay
   quiet). Add a row to the lint table in `spec/diagnostics.md`.
3. Implement the rule under `crates/cpl-lint/src/rules/`, one family per
   file. The registry is generated from `lint.toml`, so there is nothing
   to register by hand.
4. Run `cargo test -p cpl-lint`. The record tests check that `before.cp`
   fires, the others don't, and the fix holds. The corpus tests check that
   idiomatic code stays clean.
5. If the rule fires on the corpus or `std`, fix the rule or add the shape
   to `near_miss.cp`. Lower a corpus baseline only for a real finding that
   the owning domain has agreed to fix.

## Retiring a lint

Retire a lint when a checker error with the same fix covers every case. Move
its `before.cp` into that error's tests, delete the record and the rule,
and keep the code reserved. A retired code is never reused.

## Checklist

- [ ] The record has all five files, and `after.cp` passes `cpl check`.
- [ ] `near_miss.cp` covers every look-alike shape found in the corpus.
- [ ] The rule reads checker facts and resolved names, not spellings.
- [ ] The fix clears the lint, adds no error and keeps output, or the
      record says why there is no fix.
- [ ] The help gives the exact replacement in the spec's vocabulary.
- [ ] `cargo test -p cpl-lint` passes, and corpus and `std` warning counts
      did not go up.

## Measures

Every quality above has a metric and a suite in the quality register of
`handbook/domains/lint-fmt.md`. Current numbers are in
`plan/domains/lint-fmt/metrics.toml`.

## Changes

- 2026-11-23: first version, from `plan/domains/lint-fmt/README.md`, `spec/tools/fmt-lint.md` and the record tests.
