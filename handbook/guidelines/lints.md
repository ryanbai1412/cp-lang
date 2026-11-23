# Writing lints

| | |
|---|---|
| Kind | Guideline |
| Owner | lint-fmt |
| Describes | `crates/cpl-lint/`, `spec/lints/`, `scripts/new-lint.py`, `cp.toml [lints]`, `with lint(...)` |

A lint flags legal code that is probably wrong, slow or not the CP Lang
way, and says how to write it instead. Read this before you add, change or
retire one. The bar: a strong contestant reads the warning and is glad it
fired.

## What a good lint looks like

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

The message names what the code does. The help gives the exact replacement
and the reason in one line. `cpl fix` rewrites it to the record's
`after.cp`, and a second run finds nothing:

```
let raw = [3, 1, 2]
let doubled = [2 * x for x in raw]
print(doubled)
```

A bad lint matches `.len` or `max` as text, so it fires on a user's own
`fn max`. It guesses that `d` is a float because its name is `d`, or it
repeats an error the checker already reports at the same place.

## The ideas behind the rules

**Precision before recall.** A lint that cries wolf teaches users to
ignore every lint. When a rule can't be sure, it stays silent or warns
without a fix, and idiomatic code (the corpus, `std/`) gets no warnings.

**The checker knows; the lint asks.** Types, what a name resolves to,
ranges and effects come from the checker's analysis, the same one the
editor holds. A lint never re-derives them from syntax, and if the checker
already reports a shape, the fix belongs on that error instead.

**Fixes you can apply without reading them.** A fix clears its lint, adds
no error, keeps the output and is already formatted. When the right edit
depends on intent, the lint gives help instead.

**Every lint enforces a written rule.** Each lint cites the checklist rule it
enforces, such as `idiomatic-code/R5` (bounds from the statement are types).
A lint with no rule behind it is a one-off: write the rule, or drop the
lint.

**Cheap and easy to turn off.** A rule is one pass over facts the editor
already has, and one line in `cp.toml` or `with lint(...)` turns it off.

Checker lints (`L06xx`) and bound lints over declared ranges (`L03xx`)
also follow `handbook/guidelines/errors.md`.

## Where to start

- **Add a lint:** run `scripts/new-lint.py`, then follow
  [Adding a lint](#adding-a-lint).
- **A lint is wrong:** add the code to the record's `near_miss.cp`, then fix
  the rule under `crates/cpl-lint/src/rules/`.
- **A checker error now covers a lint:** see [Retiring a lint](#retiring-a-lint).

# Reference

The full rules, each with a reason, an example and what enforces it, are in
`handbook/guidelines/lints-checklist.md`.

## Adding a lint

1. `python3 scripts/new-lint.py <name> --group idiom --level warn` takes
   the next free code and creates `spec/lints/<name>/` with `lint.toml`,
   `before.cp`, `after.cp`, `near_miss.cp` and `explain.md`. Land the record
   soon after, since two lanes can take the same code.
2. Write `before.cp` (fires once), `after.cp` (what the fix writes; it must
   pass `cpl check`) and `near_miss.cp` (every similar shape that must stay
   quiet). Set `rule` in `lint.toml` to the checklist rule the lint enforces.
   Add a row to the lint table in `spec/diagnostics.md`.
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
- [ ] `lint.toml`'s `rule` names the checklist rule the lint enforces.
- [ ] `near_miss.cp` covers every look-alike shape found in the corpus.
- [ ] The rule reads checker facts and resolved names, not spellings.
- [ ] The fix clears the lint, adds no error, keeps output and converges,
      or the record says why there is no fix.
- [ ] The help gives the exact replacement in the spec's vocabulary.
- [ ] `cargo test -p cpl-lint` passes, and corpus and `std` warning counts
      did not go up.

## Measures

Every quality above has a metric and a suite in the quality register of
`handbook/domains/lint-fmt.md`. Current numbers are in
`plan/domains/lint-fmt/metrics.toml`.

## Changes

- 2026-11-23: first version, from `plan/domains/lint-fmt/README.md`, `spec/tools/fmt-lint.md` and the record tests.
