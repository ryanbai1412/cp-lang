# Tech-debt

| | |
|---|---|
| Kind | Domain charter |
| Owner | tech-debt |
| Status | Active |
| Describes | `scripts/debt-ratchet.py`, `scripts/test_debt_ratchet.py`, `scripts/check-handbook.py`, `plan/domains/tech-debt/` |

The tech-debt domain keeps the codebase simple enough that an everyday
change touches one obvious place. It measures the shape of the code, makes
every measure of debt shrink-only, reviews what other domains land, and
builds the checks that hold every domain to its own quality promises. If
you have added a std method or a lint and found exactly one place to put
it, that was this domain's work.

## What good looks like

A contributor adds a std function. They write it in `std/`, with doc
examples, and nothing else: the checker, the interpreter, the C++ back end
and the editor all pick it up, because none of them knows std functions by
name. Their commit runs one gate. If it copies a function body that exists
elsewhere, or adds a crate dependency the architecture doesn't allow, the
gate says so with the exact file and line. If the commit also changes saved
expected output, the hourly review reads why.

A week later the codebase is a little smaller than it was: one interpreter
engine instead of two, one output comparison instead of three, and the
counts that measure each of these can't rise again.

## How it fits together

- **The debt ratchet** (`scripts/debt-ratchet.py`) measures structural
  debt from the tree: tree-walker fallbacks, std names matched in the
  compiler, duplicated function bodies, files over the size cap, crate
  edges, skip lists. Each count is compared with a baseline that may only
  go down. It runs inside the normal gate, so there is no separate step.
- **Accountability checks** grade the other domains' quality registers:
  every row must name a real check, rules cite their guideline, escaped
  bugs are traced to the row that should have caught them, and mutation
  testing shows whether a suite catches the bugs it claims to.
- **Reviews** read what landed: hourly for duplicated mechanisms and
  privilege by name, and a cold read of each domain's design every few
  hours. Findings go to the owning domain, which fixes the cause.
- **Cross-domain moves** that no single owner can make, such as deleting a
  duplicate mechanism used by four crates, land here as one change.

## How we know it is good

The counts in `python3 scripts/debt-ratchet.py --show` go down over time
and never up, and the near-duplicate and re-derivation detectors find
little that is new. The weakest areas today are the second interpreter
engine, which still runs part of every traced program; std functions the
IR lowers by name; and the register rows across all domains that name no
check yet.

## Where to start

- **A ratchet failed on your commit:** run the command it prints
  (`--list <name>`) to see each entry; fix the cause, or lower the
  baseline if you removed debt.
- **Adding a new kind of check:** read `handbook/process/quality.md`, then
  add a counter to `scripts/debt-ratchet.py` with a test in
  `scripts/test_debt_ratchet.py`.
- **Finding where a fact or job lives:** `handbook/design/sources-of-truth.md`.

# Reference

## Scope

Owns:

- `scripts/debt-ratchet.py` and its baselines: every shrink-only count of
  structural debt, and the detectors that run inside it.
- `scripts/test_debt_ratchet.py`: the ratchet's own tests.
- `scripts/check-handbook.py`: the outsider-writing check for the handbook.
- `handbook/process/quality.md`, `handbook/design/overview.md`,
  `handbook/design/sources-of-truth.md`.
- The accountability checks: register rows name real checks, skip and
  allowlist sizes only shrink, commits that change expected output together
  with code are flagged, mutation scores, and the escape log.
- Cross-domain moves that no single owner can make, such as splitting a
  file over the size cap or deleting a duplicate mechanism, landed as one
  announced commit.

Does not own:

- Any crate's behavior. A structural fix inside a crate is agreed with that
  crate's owner, who keeps it.
- Language semantics (`spec/`), the stdlib's API, or performance budgets.
- The gate's tiers and time budgets (infra). This domain adds no new gate:
  every detector runs inside the existing ratchet.

## Interfaces

Provides:

- `python3 scripts/debt-ratchet.py`: fails when any count rises above its
  baseline. `--show` prints every count, `--list NAME` lists the offenders
  of one count. It runs in the fast gate in under a second.
- A burn-down of every count, published to `plan/domains/tech-debt/`.
- A cold review of landed diffs: duplicated mechanisms, one-off special
  cases, single-use helpers and layering breaks, each fixed or sent to its
  owner.

Needs:

- Each domain: a quality register in its charter whose rows name real
  checks, and a fix to its suite, not only to the bug, when a bug escapes.
- Compiler: the shared check session, the intrinsics table and the VM work
  that the ratchets measure progress against.
- Infra: the ratchet stays in the fast gate.

## Quality register

Each row is one attribute this domain must be high on, the metric, the
strongest mechanism that enforces it, and the check that runs it. `missing`
means no real check exists yet. Targets and current numbers are in
`plan/domains/tech-debt/metrics.toml`.

| Attribute | Metric | Mechanism | Check |
|---|---|---|---|
| One engine runs programs | Statements the tree-walker runs, kinds of VM fallback | Re-architecture: delete the tree-walker; meanwhile a shrink-only fallback list | `crates/cpl-cli/tests/vm_fallbacks.rs`, `scripts/debt-ratchet.py` (`vm_fallbacks`, `tree_walker`) |
| No stdlib privilege by name | Std names matched in the checker, IR, lints and backend | Re-architecture: an intrinsics table; meanwhile ratchets | `scripts/debt-ratchet.py` (`ir_builtin_names`, `backend_std_names`, `lint_std_names`, `prelude_privilege`) |
| One home per mechanism | Near-duplicate and duplicate fn bodies across crates | Ratchet with a detector | `scripts/debt-ratchet.py` (`near_dup_fns`, `dup_fn_bodies`, `lint_dup_helpers`) |
| Consumers use checker facts, not their own | Re-derivations of types or facts outside the checker | Ratchet with a detector | `scripts/debt-ratchet.py` (`consumer_rederived`) |
| One analysis per process | Query databases built outside the shared session | Re-architecture: one check session; meanwhile a ratchet | `scripts/debt-ratchet.py` (`own_databases`) |
| Layering holds | Crate edges not in the allowed list, layer violations | Gate on new edges | `scripts/debt-ratchet.py` (`crate_edges`, `dev_crate_edges`, `layer_violations`) |
| Files stay readable | Rust files over the size cap or within 5% of it | Ratchet | `scripts/debt-ratchet.py` (`rs_near_cap`) |
| One entry point per job, same result everywhere | CLI, editor and playground disagreeing on check, fix, run or trace | Differential tests | `crates/cpl-wasm/tests/parity.rs`, `crates/cpl-cli/tests/lsp_parity.rs` |
| Generated C++ copies only when a reason requires it | Deep copies and `std::function` in emitted C++ | Ratchet | `scripts/debt-ratchet.py` (`cpp_deep_copies`, `cpp_std_functions`), `crates/cpl-cli/tests/read_only_copies.rs` |
| Side effects behave the same in both engines | Corpus variants whose interpreter and C++ output differ | Generated differential variants | `scripts/debt-ratchet.py` (`diff variant/*` run) |
| The ratchet measures what it claims | Ratchet unit tests | Hand-written suite | `scripts/test_debt_ratchet.py` |
| Script tests run somewhere | Python test files no gate runs | Ratchet | `scripts/debt-ratchet.py` (`unrun_script_tests`, `direct_python`) |
| Docs point at code that exists | Paths named in architecture docs that are gone | Ratchet | `scripts/debt-ratchet.py` (`stale_arch_paths`) |
| Handbook reads for an outsider | Shared-history phrases, commit hashes | Lint | `scripts/check-handbook.py` |
| Skipped tests don't grow | Ignored Rust tests, pending tests | Ratchet | `scripts/debt-ratchet.py` (`rust_ignore`, `pending_tests`) |
| Allowlists, xfails and known-difference lists don't grow | Entries across every such list | Ratchet | `scripts/debt-ratchet.py` (`skip_lists`) |
| Every register row names a real check | Rows naming no existing test, command or job, across every domain | Ratchet | `scripts/debt-ratchet.py` (`register_rows`) |
| Every rule has one home | Lints and gates citing no handbook guideline heading | Count, gated once domains fold their rules ([quality.md](../process/quality.md#rules-live-in-guidelines)) | `scripts/debt-ratchet.py` (`uncited_rules`) |
| Expected output changes for a reason | Commits that edit expectations and code together | Post-land detector, read in the hourly review | `scripts/expectation-edits.py` |
| Suites catch the bugs they claim to | Mutation score of `cpl-check` and `cpl-lint` suites | Mutation testing on the code each change touched | `scripts/mutation-score.py` |
| Escaped bugs fix the suite | Escapes whose suite isn't fixed, or that name no register row | Escape log, checked by a count | `plan/escapes.toml`, `scripts/debt-ratchet.py` (`escapes_open`) |
| Every feature works in every tool | Features (syntax forms, std items) with no case in the checker, VM, C++ backend, trace, fmt, lint, LSP or docs | Generated coverage matrix | `missing` |
| Every builtin with two engine paths agrees at its edges | Builtins without an interpreter-vs-C++ case at negative, empty, overflow and boundary inputs | Generated differential cases | `missing` |
| Stdlib privilege gone, not hidden | Std items that compile only because the compiler knows their name | Re-architecture: build std as a user library in a test | `missing` |
| Facts are generated, not written twice | Hand-written lists (lints, crates, metrics, std items) that a generator could produce | Ratchet with a detector | `missing` |
| Generated files are fresh | Committed generated files that differ from regenerating them | Regenerate and diff in the gate | `scripts/regen.sh` |
| Decisions stay findable | Decision numbers used twice | Ratchet | `scripts/debt-ratchet.py` (`decision_dup_numbers`) |
| No dead language era | References to the removed 0.2 edition | Ratchet | `scripts/debt-ratchet.py` (`edition_refs`) |
| No unfinished std | Std functions whose body is a stub | Ratchet | `scripts/debt-ratchet.py` (`stdlib_todo`) |
| Dead code is deleted | `allow(dead_code)` attributes; items no caller reaches are not yet counted | Ratchet | `scripts/debt-ratchet.py` (`dead_code_allows`) |
| Suites cover the code they own | Line and branch coverage per crate | Coverage matrix | `missing` |
| Tests are deterministic | Tests that pass and fail on the same tree | Rerun-on-fail detector | `missing` |
| Few, vetted third-party crates | Third-party packages in `Cargo.lock` | Ratchet: a new one fails the gate until `--update` and a `decisions.md` line | `scripts/debt-ratchet.py` (`external_crates`) |
| An everyday change touches one obvious place | Files, crates and minutes for a cold agent on a fixed change | Process: cold change-cost probes | process |
| Landed diffs add no new debt | Duplicates, special cases, single-use helpers per review | Process: hourly review of landed diffs | process |

## Suites and tools

| Tool | Catches | Run |
|---|---|---|
| Debt ratchet | Any structural count rising above its baseline | `python3 scripts/debt-ratchet.py` (`--show`, `--list NAME`) |
| Ratchet tests | A detector miscounting | `python3 -m unittest scripts/test_debt_ratchet.py` |
| VM fallback list | A program part newly running on the tree-walker | `cargo nextest run -p cpl-cli --test vm_fallbacks` |
| Walked-statement counts | Tree-walker statements, including inside fallbacks | `CPL_VM_STATS=1 cpl run ...` |
| Entry-point parity | CLI and playground disagreeing | `cargo nextest run -p cpl-wasm --test parity` |
| Handbook check | Text only its writer can follow | `python3 scripts/check-handbook.py` |
| Mutation score | Code changes no test notices | `python3 scripts/mutation-score.py -p cpl-lint --since <rev>` |
| Expectation edits | Commits that change saved output beside the code that makes it | `python3 scripts/expectation-edits.py --since "1 hour ago"` |

A count goes down by fixing the cause and re-recording the baseline in the
same commit. A count never goes up: a change that needs more first removes
the same amount elsewhere, or records why in `decisions.md`.

## Changes

- 2026-11-23: first draft, folding in the stable parts of
  `plan/domains/tech-debt/README.md` and `ARCHITECTURE.md`.
- 2026-11-23: prose overview first; scope, interfaces and register under Reference.
