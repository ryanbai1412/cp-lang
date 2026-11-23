# Architecture & tech-debt: plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order

0. **Milestone 1 first (2026-11-22).** Ordered by what blocks the
   [acceptance scenario](../../product/MILESTONE-1.md):
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
   also edits `plan/ARCHITECTURE.md`. Hook it into the gate step that already
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
5. **Keep `plan/ARCHITECTURE.md` true.** Rewrite part B1 from `debt-ratchet.py --show`
   output; every path it cites is checked by `stale_arch_paths`.

## General fixes owed

| Instance (local fix) | General fix | Owner |
|---|---|---|
| Checkpoints only at main's `while`/range-loop back edges; a run inside one call takes none (`cpl-interp/src/checkpoint.rs`) | An explicit VM frame stack (no Rust recursion per CPL call), so a checkpoint is the frame stack plus globals at any back edge or call | compiler (interp) |
| Free `count`/`all`/`any` of a generator counted elements: std.seq's fold shadowed the core reduce; a name-and-shape guard keeps the core form (3f8186fcf, decision 323) | No core reduce beside a std fn of the same name: std declares the generator form itself (`count(g: Gen<Bool>)`), so overloads, not checker/lowering guards, pick it. Folds the binary `min`/`max` exception (R11.3.3) into the same mechanism | compiler + stdlib |
| `print(V.count)` above `domain V = 0..5` printed 0: the checker hoists constant items, lowering ran them in source order (3fabf485b, decision 324) | One item order computed by `cpl-check` (which items hoist, in what order, at every block level) that lowering consumes, so the two can never disagree; covers local blocks and `type`/`const` too | compiler |
| M1 P0 `map-get-copies-list`: `if let l = m.get(k)` copied the mapped list every lookup; when the map is a plain variable nothing changes while the pattern's temporary lives (`place_stays`), the C++ binds `cpl::map_lend`, a pointer into the entry; `let l = m.get(k) ?? d` lends the same way (`let_lending_coalesce`) | Every optional-valued read of a place lends whenever `place_stays` holds: `MapGet` also as a `for` iterable (`for v in m.get(k) ?? []`, a constant-factor copy) and a named `let o = m.get(k)`, and `First`/`Last`/`SetMin`/`SetMax`/`FirstGe`.. of non-scalar elements, through one `Lent<V>` path in codegen instead of one builtin at a time | perf (codegen) |
| 9 of the 11 "silent wrong answers" in conformance were spec cases written ahead of the implementation, or wrong expectations, not regressions | A case for an unimplemented rule carries `// pending: <owner>`, and `cpl-conform` reports pending cases apart from failures, so a wrong answer on a supported feature is always a real bug | spec + infra |

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
