# Resume: CP Lang spec + conformance-coverage owner

## Goal and done-bar
Own `spec/` (normative) and grow `spec/conformance/` so the pass rate measures
spec coverage, not just hand-picked cases. Compiler and std bodies are owned
by the implementation lane: never edit `crates/` or `spec/std` bodies to make
a case pass, and never delete or weaken a case.

The work is done when:
- every R-rule has an accept case, and a reject case wherever the rule forbids or traps;
- every spec example runs (core code blocks and std `/// example:` lines);
- the feature x consumer matrix has no empty cells;
- every trait method is called on every type that claims the trait;
- all of the above are gates in `scripts/check.sh` that can only ratchet up.

## Measure
- `cargo build --release && ./target/release/cpl-conform all --check`: suites, plus a ratchet against `research/scoreboard.json`.
- `python3 scripts/conformance-coverage.py`: rewrites `spec/conformance/COVERAGE.md` (per-chapter accept/reject/complete %) and `spec/conformance/rule-gaps.txt`. `--check` fails if the report is stale or a gap is added.
- `python3 scripts/conformance-matrix.py`: regenerates `spec/conformance/matrix/`. `--check` fails if it is stale or a cell has neither a case nor an n/a reason. Only on the wip branch.
- `./target/release/cpl-conform conformance --verbose <filter>`: a focused run.

## State (2026-11-22 11:00 PST)
- About 1700 conformance cases on main, roughly 94% passing; failures are compiler gaps, recorded in the ratchet.
- Rules (528): 94% have an accept case and 79% of forbidding rules have a reject case. 86% are complete; 70 gaps are listed in `rule-gaps.txt`.
- Matrix (wip branch): 3 matrices, 211 generated cases and 22 n/a cells; 52 cases fail.
  - Index types x consumers: 46 of 121 fail.
  - Optional producers x unwrap form x scope: 3 of 75 fail.
  - Generic types x how args are given x call kind: 3 of 15 fail.
- Trait generator (3): not started. The design is below.

## Landed (origin/main)
efca1ba4, c99edd21, ade499ee, d02c3d34, f0740f24 and later trains: about 650 cases, the tool header, and the rulings in decisions 211, 213, 216 and 217.
Last in flight at wind-down: 32a81dd9 (empty subsets, compose order, grid Prefix) and 406a52f8 (let-else, domains as Seq, static type args). Check `git log origin/main`.

## In flight: branch `devin/wip-conformance-matrix`
- `scripts/conformance-matrix.py`, `spec/conformance/matrix.toml` and the generated `spec/conformance/matrix/`. Works and is deterministic; the gate step is in `check.sh`.
- `research/conformance-generators/spec-blocks-given.patch`: unbuilt change to `crates/cpl-conform/src/spec_blocks.rs`. It runs bare fences in `spec/core` and prepends a `<!-- given: ... -->` prelude. Untested; with it, 184 of 210 blocks failed on missing names.
- `research/conformance-generators/matrix-fails.txt`: the failing matrix cells, with their errors.

## Next steps (priority order)
1. Land the matrix branch.
   - Test: `scripts/check.sh --fast` passes and `conformance-matrix.py --check` exits 0.
   - The failing cells are already known to the ratchet, since `all --check` records new cases as known-failing.
2. Trait generator (3): `scripts/conformance-traits.py`.
   - Parse `pub trait` bodies in `spec/std/*.cp`, plus the trait block in `spec/core/12-powers.md` (Seq, Keyed, Sized, Bidir, Indexed, IndexedMut, Contains).
   - Parse claims from `impl Trait for Type:` lines (about 72 in std).
   - Add core claimers by hand in `spec/conformance/traits.toml`: List, Str, `[D]T`, Range, and domains as Seq/Bidir.
   - Each (type, trait, method) needs a snippet and expected output in `traits.toml`. A missing one is a gap in `trait-gaps.txt`, and that count may only fall.
   - Test: `--check` fails when a trait method is added without a snippet or a gap entry.
3. Spec-example generator (1): finish the `given` patch, or mark the remaining blocks spec-only.
   - Test: every core block is a case or spec-only, and every `/// example:` runs. The docs suite already runs the 2,683 examples.
4. Close the 70 rule gaps. Most need a reject case; the R03, R09 and R13 chapters are thinnest.
5. Send each new failure to the implementation lane as a list of case paths, never as compiler fixes.

## Gotchas
- `// expect: E#### at N` counts N from the first line after the `// expect:`/`// rule:` headers, not from the top of the file.
- R10.3.3 bars printing a Bool, so use `print(yes(b))`.
- `{}` is an empty block, not an empty Subset; use `Subset<P>.empty` (decision 217).
- `V.count` is a property, while `V.count(f)` is the Seq method (R04.2.20).
- A position's integer form is `Int(d)`; no `.ord` exists, and `core-04/enum-position-has-no-ord.cp` pins that.
- `.at(k)` takes a key and `.nth(k)` is a walk offset (decision 213). `.collect()` and `List(w)` count from 0.
- The pre-commit hook needs `target/debug/cpl` in the checkout. A git worktree needs its own build or a `target` symlink.
- `git commit -F -` from a backgrounded subshell loses stdin; use `-m` instead.
- `land.sh` can wait a long time on the land lock; another lane's train carries your commits.
- Python is 3.10, which has no `tomllib`; `conformance-matrix.py` has its own small TOML parser.
- Don't use `stdlib/`, `CPL_SPEC`, `CPL_EDITION` or `CPL_STD`; v0.2 is deleted.

## Open questions for Ryan
- Subset<B> and Pair<V> as domains: they have no print form, and the R04.3 neighbour, cut and `Int` members are marked n/a in the matrix. Should they get them?
- No diagnostic code exists yet for a struct field with a runtime bound.
- R11.4.2 says builds pin a compiler, but no spec names the `cpl build` option for it.
- Should `V.count(f)` exist alongside the `V.count` property? It is ruled yes in R04.2.20, but the overload is unusual.
