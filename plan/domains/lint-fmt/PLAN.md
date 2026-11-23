# lint-fmt plan

The detail behind [README.md](README.md). Target design:
[design/overview.md](design/overview.md). Numbers: [metrics.toml](metrics.toml).

## Workstreams, in order
Milestone 1 (`plan/product/MILESTONE-1.md`) sets the order: what a newcomer
meets in the browser and the editor (Neovim/Vim, VS Code) first,
architecture after. Deadline ~09:30 PST 2026-11-23; hours, not days.

1. **Lints in the playground (M1, with site).** Landed: `cpl_lint::lint_analysis`
   (shared with the LSP) and `Playground::lint` / `cpl_lint()`, tested on
   every record. Left: site calls it from the editor's `checkLinter`.
2. **Format on save on every editor and OS (M1, with tools).** Landed: LSP
   formatting follows `cp.toml [fmt]`; VS Code defaults `editor.formatOnSave`.
   Next: Neovim plugin formats on write through the LSP, Vim hook on
   `cpl fmt --stdin`, both in the editor smoke tests; `cpl fmt` keeps CRLF
   files CRLF for Windows checkouts. Fmt gate stays in the fast gate.
3. **Newcomer ranking (M1).** Count `L` codes on corpus `port.cp` as the
   newcomer proxy; record it in `metrics.toml`; audit the top 10 by count:
   message names the CPL spelling, fix verified, explain teaches the idiom.
   L0414 (most hits) gets precision work or leaves the default set.
4. **Grow teaching lints by port evidence (M1).** Catalog families that fire on
   ports (C++ habits: `chmin`, `.back(); .pop_back()`, `(a % m + m) % m`,
   `YES`/`NO`), each a record with a fix where meaning-safe.
5. **Done, kept green:** fmt gate (M1 of the review packet), lint records and
   generated registry, groups and `cp.toml` levels, `[fmt]` widths, the
   false-positive ratchet, the fact-table prototype.
6. **Deferred past Milestone 1:** recognizers and library `@lint` (M3),
   `bounds.rs` onto the fact table, sorted `use` and the stamp, L0601-L0609
   levels in `cpl lint`.

### Earlier order (review packet, kept for the reasons below)
1. **Fmt gate (M1).** Land `devin/wip-fmt` (4 commits: c-spelling spacing,
   `impl`/`lends`/`without`/`as` spacing, leading-dot shorthands, `[n]T`,
   member keywords; `scripts/fmt_check.py` in the fast gate; a design note on
   line breaking). Fix the `c_operators.cp` meaning change with a regression
   case in `crates/cpl-fmt/src/tests.rs`. Then one mechanical commit formats
   corpus ports, `std`, examples and docs (layout tests exempt, per the
   question in `plan/QUESTIONS.md`).
2. **Lint as data (M2).** Land `devin/wip-idioms-lint`'s 38 `spec/lints/<name>/`
   records; generate `LINTS` from them (build script or a checked-in generated
   file with a drift test); add a test that the records and the lint table in
   `spec/diagnostics.md` agree. Behaviour unchanged.
3. **False-positive ratchet (M2).** Classify each of the 50 corpus `spec.cp`
   and 293 `std` warnings: a false positive becomes a near-miss case and a
   fix in the lint; a true finding becomes a request to corpus or stdlib. Gate
   the count in `scripts/check.sh --fast` so it only goes down.
4. **Facts from the checker (M3, needs compiler).** Contract tests in
   `crates/cpl-lint` for the facts the lints need; the compiler exposes a fact
   table; the linter and `cpl-lsp` read it; delete `rules/bounds.rs` and the
   `Proven*` passes. Until then the current analysis stays, unchanged.
5. **Recognizers and `@lint` (M3).** fold, search, build, scan first (they
   cover `rules/taste/`'s largest lints); port `hand_rolled_*`; stdlib `@lint`
   declarations replace the hard-coded names; L0601-L0609, which `cpl-check` emits, are named in
   `cpl-lint` and reported by `cpl lint` (decisions.md 239).
6. **Spelling cleanup.** `@allow` removed in favour of `with lint(...)`
   (`cpl fix` rewrites it), after the spec domain confirms.
7. **Grow the catalogue.** From the idioms catalog, P0 and bound-aware
   correctness first, each as a record plus `@lint`, opt-in until its corpus
   run has 0 false positives.
8. **Fmt reordering.** Sorted `use` runs and the line-1 stamp, each a pass
   with its own verify step. The handoff's E0206 cascade (sorting changed
   diagnostics in the old `examples/local-lib`) does not reproduce on main's
   copy, `research/archive/local-lib-v02/spec/worked.cp`: 1 E0206 before and
   after sorting; `fmt_check.py` re-checks meaning anyway.

## General fixes owed

| Instance | General fix | Owner |
|---|---|---|
| ccc99s5: `fill(0, 0, 0)` dropping a `Bool` warns (W0101) only once L0593 moves it out of the test loop | W0101 checks expression statements in every block, not only top level | diagnostics |
| L0591 `while_not_empty_pop` matches any no-argument method after `not q.is_empty`, trusting that its `None` means empty (naming `pop` would add a std name to the rules) | The library declares the contract (`@lint` on `pop`/`pop_front`/`pop_first`: "returns `None` iff empty"), and the rule reads it | lint-fmt (M3 library `@lint`) |
| The `some(`/`none` migration (835de99a1) rewrote a local `none` in `cpl-cli/tests/golden.rs` to `None`, breaking main | Spelling migrations run through `cpl fix` on resolved names (a local binding is never a removed spelling), and Rust-side edits go by hand with `cargo check` | diagnostics |
| The hand-search lints (L0580–L0584) match squashed source text, so a spelling they do not list is missed | Match on the AST once the loop-forms recognizers (`design/loop-forms.md`) cover `while` loops | lint-fmt |
| `cpl fmt` chain breaks inside a block lambda's body end the statement, so the formatter keeps that file's chains whole | The lexer continues a line ending in an operator inside a layout block too (`layout_block_line`), as it does at top level; then drop the chains-off fallback in `format_lf` | compiler (syntax) |
| `trial_division` fires on `std/math.cp`'s own `factor`/`divisors` (baselined) | A rule that teaches a std fn skips that fn's own body, read from the resolved definition in the fact table | lint-fmt |
| L0566 teaches `Window(min)` because `a.windows(k)` on `[0..n]Int` fails `cpl check` (E0302 in `piece`) | `windows` checks on every `Indexed`; then L0566 names `a.windows(k).fold(min)` as std/dp.cp does | stdlib |
| `cpl fmt` broke only inside brackets, so 41 port and 40 std lines over 100 columns passed `--check` | Done for operator chains: break after the loosest operators, where the lexer continues the line, so no brackets are added. Left: 14 lines that are inline `if c: a else: b` or comprehension clauses; break a comprehension before `for`/`if` inside its bracket, and parenthesize an inline `if` to break before `elif`/`else` | lint-fmt (fmt) |
| `contains_then_add` and `copy_comprehension` only teach, because the rewrite is right only for a `Map` (or a sequence) | Read the checker's type of the place from the fact table (as `typed.rs` does for L0540/L0542) and attach the fix where it is a `Map` / sequence | lint-fmt |
| `queue_head_index` matches `.len` by name (`lint_std_names` 149 -> 150) | Match the resolved std fn through the fact table, one mechanism for every rule that names std | lint-fmt, compiler (fact table) |
| `grid_neighbour_step` (L0534) existed when I wrote a second grid-edge rule; dropped mine | Before a new rule, grep `spec/lints/*/lint.toml` summaries for the shape; a record test could reject two records whose `before.cp` fire each other | lint-fmt |
| `push_loop` and `copy_sort_writeback` both fired on one copy-sort-writeback (n33); added the pair to `SUPERSEDED_BY` | Tried dropping any finding inside a wider fix: it hid 5 distinct teachings on corpus/std (L0540 `mask.bit(i)` inside an L0504 `sum`, L0415, L0506, L0510), since the wider rewrite folds the inner fix in. Same root cause is the lint's judgement, so the table stays; the general check owed is a corpus test listing pairs that co-fire on one span, each either in `SUPERSEDED_BY` or marked distinct | lint-fmt |
| Lint codes collided twice (L0530-L0532) when two lanes added records at once | `record_codes_are_unique` fails a duplicate; lanes take code blocks (lint-fmt L054x, diagnostics L053x). Next: `cpl lint --new <name>` picks the lane's next free code | lint-fmt |
| L0521's running total, L0542 and L0504's `x = f(x, e)` match `push`, `last`, `len` and a parameter name as text | rules match resolved symbols from the fact table (callee is a fn, a parameter or a std method), so no rule reads a name | lint-fmt, with compiler for the fact table |
| L0522 knows three copy spellings (`to_list`, comprehension, push loop), each matched separately | one "value is a copy of `xs[l..r]`" recognizer shared by every rule that reads a copy | lint-fmt |
| `cpl explain` and `cpl lint --list` needed record math (`\(O(\log P)\)`) in plain text, so `cpl_lint::plain_math` handles a few macros | one LaTeX-to-Unicode fallback shared by LSP hover, CLI and the playground for std docs, spec and records; `plain_math` then calls it | site |
| L0518 took only `false`/`0` as a sieve mark, missing `comp[j] = true` | marks are any constant written at a stepped multiple | lint-fmt (done in a5317254) |

## Why each milestone
- **M1 first** because fmt is the smallest, nearly done, and every later
  bulk change (fixes, codemods, ports) is printed by it; a fmt that can change
  meaning makes every autofix suspect.
- **M2 before new lints** because 343 warnings on idiomatic code mean the
  existing lints are not trustworthy yet, and records make each lint's
  contract (fires / silent / fix) a test instead of prose.
- **M3** is the architecture: it removes the duplicate analysis and the
  stdlib privilege, and is what makes ~670 catalogue entries cheap. It depends
  on the compiler lane, so it runs in parallel with M2's ratchet.

## Alternatives considered
- **Fmt on a parsed tree (rewrite as a pretty-printer).** Would allow
  reordering and smarter breaks, but loses formatting of broken files and
  needs a full lossless tree; the token approach already holds 0
  non-idempotent files. Kept tokens; reordering gets its own verified step.
- **Keep lints in Rust, add records only as tests.** Less churn, but the
  registry, the spec table and the docs keep drifting (already 11 codes
  disagree). Rejected.
- **Linter keeps its own analysis.** Avoids a compiler dependency, but keeps
  ~3k duplicated lines and two answers to "is this index proven". Rejected.
- **All idiom lints default-on now.** Rejected by Ryan's corpus-gate rule.

## Risks
- The fact table slips in the compiler lane: M3 waits; M1, M2 do not.
- The bulk format collides with other lanes' in-flight edits to ports: land it
  as one mechanical commit at a quiet moment, announced in the corpus worklog.
- The ratchet finds that many warnings are true: then the fix is in the ports
  (corpus) or stdlib, not in lint; tracked as requests, not suppressed.
- Generated registry adds build complexity: prefer a checked-in generated file
  plus a drift test over a build script.
- `devin/idioms-docs` and `devin/wip-idioms-lint` overlap (shared commits):
  `git cherry` before landing, so nothing lands twice.

## Phase 2, first
1. Rebase `devin/wip-fmt` on main, renumber its decision if 214 is taken,
   run `scripts/check.sh --fast`, land with `CPL_LANE=lint-fmt scripts/land.sh`.
2. Regression case and fix for the `c_operators.cp` meaning change.
3. `git cherry` `devin/idioms-docs` against `devin/wip-idioms-lint`; land the
   lint records and engine part; hand `spec/idioms/*` to the idioms lane;
   move `plan/LINTER.md` to `design/linter.md`.
4. Start the ratchet: classify the 50 corpus `spec.cp` warnings.
