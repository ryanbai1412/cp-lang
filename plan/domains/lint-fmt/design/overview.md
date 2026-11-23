# lint-fmt design overview (2026-11-22)

What v0.3's `cpl lint` and `cpl fmt` are. One page; the normative rules stay in
`spec/tools/fmt-lint.md`. Builds on the linter design Ryan reviewed this round
(`@lint` next to the API: yes; idiom lints warn by default once they pass the
corpus gate).

## Target
**Fmt.** One layout, no options: 4 spaces, 100 columns, one blank line,
comments kept, an over-wide bracket breaks one element per line with a
trailing comma. Token-preserving by construction (verify re-lexes the output),
so it formats broken files at save time. v0.3 adds sorted `use` runs and the
line-1 stamp, each with its own verification.

**Lint.** One engine, two kinds of lint, data-driven:

```text
source --parse--> syntax tree --check--> typed IR + facts --> lint engine --> findings + fixes
                    |                         |                 ^
                    +---- syntactic lints ----+                 |
                                    lint records + @lint (data) +
```
- **Syntactic lints** (spellings, removed syntax) run on the tree, need no types.
- **Semantic lints** run on typed IR and read the checker's **fact table**
  (ranges, proven indexes, domain sizes, trip counts, call costs), the same
  table LSP hints and `cpl explain` read. The linter does no analysis of its own.
- **Shape recognizers** classify loops once per file (fold, search, build,
  scan, bisect, window, union-find, sieve, lifting, copy-writeback, sentinel,
  index-space); lints match shapes, so `for i in 0..n` and `while` agree.
- **Libraries declare lints:** `@lint(name, from: <shape or pattern>, to:
  "<replacement>", tier: fix)` beside the API; user libraries get the same.
- **One record per lint** in `spec/lints/<name>/` (`lint.toml`, `before.cp`,
  `after.cp`, `near_miss.cp`, `README.md`); the Rust registry, the lint table in
  `spec/diagnostics.md`, site pages and `cpl explain` are generated from it.
- **Fix tiers:** `fix` (applied by `--fix`), `fix?` (review), `warn`, `doc`.
  A fix is kept only if the fixed file checks; tests require before/after to
  agree in interpreter and C++, and stress for `fix?`.
- **Levels:** groups `correctness`, `perf`, `idiom`, `style`, `pedantic`;
  `cp.toml [lints]` by group or name; in code only `with lint(name: allow)`.
- **One owner per finding.** What the checker already reports (a getter
  called with `()`, a renamed API) is a checker diagnostic with a fix, never
  also a lint. Today 7 lints fire only on code the checker rejects (L0205,
  L0206, L0512, L0513, L0515 say the same thing; L0510 `by_twin` and L0511
  `tree_order` name a fix the checker's error lacks); they retire into the
  checker after this review (worklog.md, 2026-11-22).
- **Surfaces:** CLI, LSP code actions, playground (the checker builds for
  wasm32 and `cpl.wasm` uses it; the prover stays native for size), stress
  reports, the skill eval.

## Non-goals
- Style options or per-project formatting.
- Lints that need whole-program or super-linear analysis.
- Lints that fire when unsure: silence beats a false positive.
- Owning which idioms exist: the idioms domain does; we turn entries into records.

## Acceptance tests
1. `python3 scripts/fmt_check.py`: 0 not idempotent, 0 meaning changes, 0
   bugs over every tracked `.cp` file, in `scripts/check.sh --fast`.
2. `cpl fmt --check` clean on corpus ports, `spec/std`, examples, docs snippets.
3. Per record: fires on `before.cp`, silent on `near_miss.cp`, `--fix` turns
   before into after byte for byte, both agree in interpreter and C++.
4. `cpl lint` over `corpus/problems/*/spec.cp` and `spec/std`: 0 warnings
   (the ratchet only goes down).
5. Registry, `spec/diagnostics.md` and records agree (a test fails otherwise).
6. A user-library `@lint` fires and fixes in a test; `crates/cpl-lint` names no
   stdlib API.
7. Same findings from CLI, LSP and wasm on the corpus.
8. Lint and fmt of the largest corpus file each under 50 ms.

## Deep dives
None yet; `plan/LINTER.md` from `devin/wip-idioms-lint` becomes
`design/linter.md` when it lands in phase 2.
