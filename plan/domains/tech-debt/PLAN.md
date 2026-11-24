# Architecture & tech-debt: plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order

0. **Milestone 1 first (2026-11-22).** Ordered by what blocks the
   [acceptance scenario](../../../handbook/product/milestones.md):
   (a) builtin divergence: for each name in `ir_builtin_names` and
   `backend_std_names`, find its interpreter and C++ implementations, add
   `cpl-conform diff` cases at the edges, fix behaviour-free mismatches or
   route the rest to compiler/stdlib with the failing case; `choose2` was one
   (decision 266 era: compiler said `choose2(-3) == 6`, std said `0`);
   (b) a user-library test in the fast gate that uses every std power
   (`--list prelude_privilege`), each failure filed with its case;
   (c) the third-party crate verdict, crate READMEs, and the remaining layout
   moves, each only when it does not collide with a milestone lane.
   Workstreams 1-5 below continue only where they serve (a)-(c) or are cheap.

1. **Land what is written (M1).** The restructure and 0.2 deletion exist as
   branches: `devin/drop-edition` (edition refs), `devin/v02-ports-delete*`
   and `devin/v02-delete*` (corpus and tests), `devin/wip-split-vm`,
   `devin/repo-layout-doc`. Rebase each on `main`, run
   `scripts/check.sh --fast`, land with `CPL_LANE=tech-debt scripts/land.sh`,
   one logical change per commit. Then the moves still only planned: `std`
   -> `std/` (with every path reference, `spec_std_refs` -> 0),
   `crates/cpl-prover` -> `crates/cpl-prover` (the package already has that name),
   uv for every Python call, `just gen --check` for generated files. For the
   57 branches with unlanded commits: those owned by another lane go to that
   lane's worklog; the rest are landed or deleted with the `git cherry` output
   in the worklog.
   The `std -> std/` move, measured on 2026-11-22: `spec_std_refs` is 300,
   most of them prose (54 in `plan/domains/stdlib`, 41 in the generated
   `skills/cp-lang`, 25 in `plan/domains/lint-fmt`, 21 in `plan/domains/spec/delta`). Shipped
   code finds the directory in 7 places: `crates/cpl-check/build.rs:12`,
   `crates/cpl-check/src/db.rs:1248` and `:2212` (which already accepts `std/`),
   `crates/cpl-check/src/prelude.rs:26`, `crates/cpl-lint/src/rules/imports.rs:20`,
   `crates/cpl-lsp/build.rs:8`, `crates/cpl-lsp/src/uri.rs:29`. Order:
   (a) each owner names the path once, a constant per crate (requests in the
   compiler and lint-fmt worklogs); (b) one tech-debt commit runs
   `git mv std std` and updates the code and scripts, with a note in each
   touched lane's worklog; (c) regenerate `skills/cp-lang` and rewrite the
   prose. The counter reaches 0 at (c); conformance must stay at 1661/1723.
2. **Ratchets (M2; landed 2026-11-22).** Fold the counts into `scripts/debt-ratchet.py` (one
   script, one baseline, its unit tests in `scripts/test_debt_ratchet.py`) and
   ratchet: files over the cap (no new allowlist entry), `edition_refs`,
   `ir_builtin_names`, `backend_std_names`, `dup_fn_bodies`,
   `layer_violations`, `tree_walker`, `direct_python`; `stale_arch_paths` is
   counted and becomes gated once round 1 findings are fixed.
   The crate-edge list becomes a baseline that may only grow in a commit that
   also edits `handbook/design/architecture.md`. Hook it into the gate step that already
   runs the ratchet, so infra's gate time is not affected (0.4 s today).
3. **Cold reviews and probes (M2, M3).** Rotate through the 12 lanes, two per
   week of work: read the lane's landed diffs since its last review, score the
   five rubric questions, write each finding with a file reference into the
   lane's `worklog.md`, log the scores here. Run probes P1-P4 cold once for a
   baseline and again at M3.
4. **Proposals to owners (M3).** Each top-10 item that is not this lane's
   goes as a concrete proposal (shape, steps, exit test) into the owner's
   worklog: one VM (compiler), builtin floor and `@intrinsic` (compiler +
   stdlib), one bound analysis (lint-fmt + compiler), std names out of the
   back ends (compiler + perf), one member catalog (compiler + tools), wasm
   lints (lint-fmt). This lane does the pure-move file splits (decision 267).
5. **Keep `handbook/design/architecture.md` true.** Rewrite part B1 from `debt-ratchet.py --show`
   output; every path it cites is checked by `stale_arch_paths`.

## General fixes owed

| Instance (local fix) | General fix | Owner |
|---|---|---|
| Checkpoints stand at `while`/range-loop back edges in any VM frame, but not where a call shares its statement with another effect, in the body of a feed loop, or in closures with `var` parameters; the VM mirrors its frames as data while still recursing on the Rust stack (`cpl-interp/src/checkpoint.rs`) | The VM runs calls on its explicit frame stack (no Rust recursion per CPL call), so a checkpoint is that stack plus globals at any back edge or call, and restore resumes the VM itself | compiler (interp) |
| Free `count`/`all`/`any` of a generator counted elements: std.seq's fold shadowed the core reduce; a name-and-shape guard keeps the core form (3f8186fcf, decision 323) | No core reduce beside a std fn of the same name: std declares the generator form itself (`count(g: Gen<Bool>)`), so overloads, not checker/lowering guards, pick it. Folds the binary `min`/`max` exception (R11.3.3) into the same mechanism | compiler + stdlib |
| `print(V.count)` above `domain V = 0..5` printed 0: the checker hoists constant items, lowering ran them in source order (3fabf485b, decision 482) | One item order computed by `cpl-check` (which items hoist, in what order, at every block level) that lowering consumes, so the two can never disagree; covers local blocks and `type`/`const` too | compiler |
| M1 P0 `map-get-copies-list`: `if let l = m.get(k)` copied the mapped list every lookup; when the map is a plain variable nothing changes while the pattern's temporary lives (`place_stays`), the C++ binds `cpl::map_lend`, a pointer into the entry; `let l = m.get(k) ?? d` lends the same way (`let_lending_coalesce`) | Every optional-valued read of a place lends whenever `place_stays` holds: `MapGet` also as a `for` iterable (`for v in m.get(k) ?? []`, a constant-factor copy) and a named `let o = m.get(k)`, and `First`/`Last`/`SetMin`/`SetMax`/`FirstGe`.. of non-scalar elements, through one `Lent<V>` path in codegen instead of one builtin at a time | perf (codegen) |
| The VM proves `FromOrd` positions per chunk (`Compiler::in_dom`, range variables over `0..len(D)`) to bring stdlib's library-domain walk level with the builtin `Subset` loop | Lowering marks a `Cast::FromOrd` it has proven in range (a walk's `nth(k)` over `0..count`, a re-cast of a checked position) as unchecked in the IR, so every backend skips the check without re-deriving it, and the C++ drops it too | compiler |
| 9 of the 11 "silent wrong answers" in conformance were spec cases written ahead of the implementation, or wrong expectations, not regressions | A case for an unimplemented rule carries `// pending: <owner>`, and `cpl-conform` reports pending cases apart from failures, so a wrong answer on a supported feature is always a real bug | spec + infra |
| A traced chunk's call (`Op::CallTraced`) runs the callee through the tree-walker's `call_at`, which tells the tracer it entered and left; by-value arguments and one-register results only | The VM's own call ops tell the tracer themselves when the chunk is traced, so there is one call path in both modes and `CallTraced` goes away | tech-debt |
| A dev `cpl` read a stale built-in std after the folder it was built from moved: `build.rs` doesn't track std/ in dev builds, and `modules::source_std` falls back to that copy silently | A dev build has no built-in std copy (or fails, naming the folder) when its source std/ is gone, so a stale std can never be read quietly | compiler |

## Why these milestones

- **M1 first** because the work is already written and every week unlanded
  costs rebase time (57 branches today) and keeps two eras of code alive.
- **M2 before M3** because without ratchets the shape regresses while twelve
  lanes land in parallel; the ratchets are what makes "no slop" checkable.
- **M3** is the architecture v0.3 promises in `AGENTS.md`: small core, no
  privileged stdlib, one mechanism per concept.

## Alternatives considered

- **Only numeric ratchets.** Rejected: duplicated concepts with different
  code (two bound analyses, two catalogs) do not show up as copied text. The
  rubric and probes cover that.
- **This lane rewrites the debt itself.** Rejected: it would collide with
  owners and land behaviour changes in code it does not own. Pure moves only.
- **Block lanes until debt is paid.** Rejected: "never block". Ratchets stop
  new debt; old debt is paid by proposals ranked by impact.
- **A crate-level dependency linter (`cargo-deny`, `cargo-modules`).**
  Deferred: a 20-line TOML read covers the edge list without a new tool.

## Risks

- Large moves (`std/`, file splits) conflict with every lane's in-flight work.
  Mitigation: land each move as its own commit early in a train and announce
  it in the affected worklogs first.
- Text-based counts can be gamed (renaming a builtin past the regex).
  Mitigation: the cold review checks the intent; the regexes are listed in
  `scripts/debt-ratchet.py` for anyone to tighten.
- `ir_builtin_names` counts string literals, so its floor (66, decision 266)
  includes aliases and type names; the classification in
  `architecture/builtin-floor.md` is the reference.
- Old branches may contain work a lane still wants; deleting needs the
  owner's confirmation in its worklog, never a silent delete.

## First action now

The builtin divergence audit, workstream 0(a), starting with `span` and the
list search functions.

## Milestone exit criteria and known debt

### Milestone 1

[Milestone 1](../../../handbook/product/milestones.md) is the priority: a Codeforces
1500+ friend understands, tries, installs, solves, tests, submits and competes
with CPL without help. Debt that causes a milestone bug comes first; cosmetic
debt waits. Constraints (Ryan, 2026-11-22): due about 09:30 PST 2026-11-23,
so steps are planned in hours with a worklog checkpoint every 2 hours; no
real judge submissions (a local Codeforces profile: compiler, flags, limits,
64 KB); Linux, macOS and Windows all supported and tested; ships on `edge`;
Neovim/Vim LSP must just work, VS Code too. This lane never blocks a milestone lane: moves land as one
announced commit, and the only gate it owns is the shrink-only ratchet.

What the milestone needs from this lane (exit criteria):
1. **No interpreter/C++ divergence from duplicated builtin semantics.** Every
   compiler builtin with its own interpreter and C++ paths has a
   `cpl-conform diff` case at its edges (negative, empty, overflow, both
   argument orders); the known disagreements (`span`, list
   `first_ge`/`first_gt`/`last_le`, `counts`) are settled in one body.
   Check: no `cpl-conform diff` failure traces to a builtin.
2. **A user library can do what std does on the milestone path.** A user
   library in the fast gate exercises each by-name power std relies on today
   (`prelude_privilege`, `backend_std_names`), or the power is listed as a gap
   with its user-reachable alternative. There is no such test today: the old
   `examples/local-lib` is in `research/archive/local-lib-v02`.
3. **Main never goes red on a stale generated file.** `scripts/gen.py --check`
   runs in the commit tier (done: decisions 294, 303).
4. **No path or line-ending assumption in shipped code breaks Windows or
   macOS.** Every path match on `"/"` strings in `crates/*/src` goes through
   `Path` components or a normalised module path, and input reading accepts
   CRLF; each one found is fixed or routed to its owner with a failing case.

Deferring: `plan/DECISIONS.md` curation, the 877 v0.2 `port.cp` files,
file-size shrinking and duplicate helpers, probe reruns, cold-review rounds
beyond findings that cause milestone bugs.

Next three steps (hours from 21:30 PST):
1. By 23:30: triage `cpl-conform diff` (1739/1879 pass, 1 mismatch, 139
   skipped) and add edge cases for the builtins with two implementations.
2. By 01:30: the Windows/macOS audit of shipped crates (path strings, CRLF,
   `/`-joined module paths); fix behaviour-free ones, route the rest.
3. By 03:30: the user-library privilege test in the fast gate. Then the
   third-party crate verdict and crate READMEs.

### Milestone 2

Most ambitious complete version: a stranger reads `handbook/design/architecture.md` and changes any
layer in its own crate; the core has no name std can't override; every generated file is
built, not hand-kept; the interpreter and C++ agree on the whole corpus with zero skips.

Exit criteria (each a command in [metrics.toml](metrics.toml)):
- `cpl-conform diff`: 0 mismatches, 0 sanitizer traps, and skips only for cases
  that expect an error (today 141 skips, mostly E0302/E0203 compile errors).
- `ir_builtin_names` at the storage, I/O, bits and numeric floor (66), every other
  name a std body within the perf bar: idiomatic p50 <= 1.1x, p95 <= 1.2x,
  max <= 1.5x, and tuned max <= 1.1x.
- `prelude_privilege` = 0: a user library can declare every power std uses.
- No file over 2,000 lines; `rs_over_cap`, `stdlib_todo` and `spec_edition` at 0.
- Cold probes P1-P4 at most 3 files each; P5 at 5/5.
- `plan/DECISIONS.md` current-only, full log archived; corpus answers derived and
  cached, committed only where underivable.

Next steps: settle `span`/`counts`/`first_ge` in one body with stdlib; split the
capped `tests.rs` files; derive corpus answers once tools' shared cache lands.

### Top 10 architectural debt (ranked, 2026-11-22)

| # | Debt | Where | Owner |
|---|---|---|---|
| 1 | Stdlib privilege: 89 names matched in lowering, 74-variant `Builtin` enum | `crates/cpl-ir/src/lower/call.rs` (13,357 lines), `crates/cpl-ir/src/lib.rs` | compiler + stdlib |
| 2 | Two interpreter engines; VM falls back to the tree-walker | `crates/cpl-interp/src/lib.rs` (`fn eval`), `vm.rs` | compiler |
| 3 | Dead 0.2 edition switch, 92 refs | `crates/cpl-check/src/edition.rs`, `check/expr.rs` (32 refs) | compiler (branch ready) |
| 4 | Two range/bound analyses; lint calls the prover directly | `crates/cpl-lint/src/rules/bounds.rs`, `crates/cpl-prover/` | lint-fmt + compiler |
| 5 | 24 files over 2,000 lines | `vm.rs` 15,367; `lower/{call,stmt,mod,expr}.rs` 6-13k | many; this lane moves |
| 6 | Library type names known to IR and emitter | `lower/call.rs`, `lower/mod.rs:24`, `cpl-codegen/src/expr.rs:2780` | compiler + perf |
| 7 | Std member catalog kept twice (checker list vs lowering match), stale rows | `crates/cpl-check/src/builtins.rs`, ignored test `crates/cpl-cli/tests/builtin_catalog.rs:32` | compiler + tools |
| 8 | Browser lints compiled out, web and CLI disagree | `crates/cpl-lint/src/lib.rs:954`, `rules/bounds.rs:680` | lint-fmt |
| 9 | Library source under `spec/`, 877 v0.2 ports, 57 stale branches | `std/`, `corpus/problems/*/port.cp` | this lane + stdlib/idioms |
| 10 | Copied helpers across crates | `cpl-{fmt,lint}/src/cli.rs` `collect` = `cpl-doc/src/cli.rs:163`; `refutable`, `prefix_shape_struct` in check and ir | tools, lint-fmt, compiler |
