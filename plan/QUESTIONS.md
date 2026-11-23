# Decision queue

Calls only Ryan makes: ones that change the language's character, and the design reviews for segment C. Everything else is an ordinary decision the owner makes and logs in `decisions.md`.

One entry per call, newest at the bottom of "Open":

```
### <domain>: <question>
Options, each with real code from a port or spec case.
Recommendation and why.
How to reverse it.
Meanwhile: the reversible default the lane works with.
```

Lanes never block on an entry: they work on the default and on everything else. When Ryan answers, the entry moves to "Answered" with the answer, and the decision goes into `decisions.md`.

## Open

### diagnostics: placeholder fix-its, or only right ones?
A broken line like `let (n, k) =` (aac4p6 `spec.cp`, recovery edit "cut the value after `=`") today gives `error[E0102]: expected a value after `=`, found the end of the line` with no fix-it.
Option A (today): fix-its only where the repair is the right one (`a && b` -> `a and b`, `!x` -> `not x`, a misspelled field -> its near name). Broken-ports fix-it rate stays 72.4%; the "cut the value" row stays at 0%.
Option B: also offer placeholders, e.g. `let (n, k) = todo`, so the program checks and runs to the hole. The rate would rise, but a fix that compiles and is wrong is the contestant's next bug.
Recommendation: A. A fix-it is a promise that applying it is correct; placeholders spend that trust, and incomplete programs already run.
How to reverse it: add placeholder fixes at the E0102 site and count them in the scoreboard; no other code depends on their absence.
Meanwhile: A; the "cut the value" row is excluded from the fix-it target.

### tech-debt: which names may the compiler know (the builtin floor)?
Today `crates/cpl-ir/src/lower/call.rs` matches 89 names, from `print` and `popcount` to library shapes like `prefix`, `first_ge`, `adjacency`, `laws_hold`; a user library cannot do the same.
Options: (a) floor = I/O (`read`, `print`, `flush`), numeric primitives (`abs`, `min`, `max`, bit ops, `sqrt`), storage of `List`/`Str`/`Map`/`Set`; each declared in `std/` with an `@intrinsic` marker any library may use; everything else written in CP Lang. (b) keep the current set, freeze it. (c) no floor, only "goes down".
Recommendation: (a), about 30 names; it is what "no privileged stdlib" means and makes every remaining special case visible in `std/`.
How to reverse it: a name moves back from `std/` to the floor list with a `decisions.md` line.
Meanwhile: `ir_builtin_names` may only go down; no new name is added without a `decisions.md` line.

### tech-debt: may the steward land pure-move refactors in other lanes' crates?
Example: splitting `crates/cpl-interp/src/vm.rs` (15,367 lines) into modules with no behaviour change.
Options: (a) yes, behaviour-free moves only, announced first in the owner's worklog; (b) proposals only, owner does every move.
Recommendation: (a); moves are mechanical, conflict-prone and cheapest when one lane batches them.
How to reverse it: owners take moves back; the steward files proposals only.
Meanwhile: (a) for `vm.rs` (agreed in the previous round), proposals for every other file.

### tech-debt: how strict are the debt ratchets?
Options: (a) hard-fail the gate on any increase; the only escape is a baseline bump in the same commit plus a `decisions.md` line; (b) warn only.
Recommendation: (a); warnings are ignored by twelve parallel lanes.
How to reverse it: switch the ratchet step to warn in `scripts/check.sh`.
Meanwhile: (a).

### site: should the landing show 0.3 programs the compiler cannot run yet?
Options: (a) keep them as tabs with a "0.3 spec" badge and no Run, as today; the syntax and library tabs show no Run button; 4 of their 8 programs now run with a sample input (`web/content/landing/syntax/records.cp`), 4 are still `spec` (`web/content/landing/library/ioi05p2.spec.cp`); (b) show only programs that run, adding the rest as the compiler catches up; (c) lead each tab group with runnable programs, spec ones after.
Recommendation: (c). The first thing a visitor clicks should run; the 0.3 shape still shows, labelled.
How to reverse it: tab order lives in `web/lib/landing.ts`; one edit.
Meanwhile: (c).

### site: should guide code blocks become live runners?
Options: (a) static highlighted blocks, as today (`web/content/docs/guides/tour.mdx`); (b) every block with status `runs` gets a compact Run and "Open in playground"; (c) only on Learn pages.
Recommendation: (b). The gate already proves which blocks run; Run turns reading into checking, which is the docs-only bar.
How to reverse it: one MDX component mapping in `web/components/`.
Meanwhile: (b), built: `remarkLive` in `web/source.config.ts` wraps each fence the gate measured `runs` in `web/components/live-code.tsx`.

### site: Learn for v0.3, breadth or depth?
Options: (a) 8 short chapters + 15 original judged problems covering a Div 2 A-C (today 3 chapters, 7 problems in `web/content/learn/read/`, `web/content/practice/`); (b) a full course to Div 1 topics; (c) freeze Learn and put the effort into guides and reference.
Recommendation: (a). It covers what a first contest needs and stays small enough to keep compiler-checked as the language moves.
How to reverse it: content only; chapters add or drop independently.
Meanwhile: (a).

### compiler: how does a product position lower?
`domain P = (R, C)`; `for (r, c) in P: if let q = P.at((Int(r) - 1, Int(c))): n += 1` (`spec/conformance/core-04/prod-at-neighbour.cp`).
Options: (a) in the IR a position of `P` is a tuple of factor positions `(R, C)`; `nth`, array offsets and `[P]T` indexing compute the row-major offset at the use. (b) a position is one packed row-major `Int`; factors are `div`/`mod` at the use.
Recommendation: (a). Neighbour steps and `(r, c)` destructuring are the common uses and stay free; storage is two small ints, narrowed by their ranges; it matches the C++ a contestant writes. (b) only wins on storage.
How to reverse it: the representation is one function in `crates/cpl-ir/src/lower/` plus the back ends' reading of it; switch there, rerun `cpl-conform all`.
Meanwhile: (a), which `devin/wip-product-domain` implements.

### compiler: one interpreter engine, no reference tree-walker?
Today `crates/cpl-interp/src/vm.rs` falls back to the tree-walker in `crates/cpl-interp/src/lib.rs` for anything it doesn't lower (13 of 13 runnable corpus programs fall back; `CPL_VM_STATS=json`), and wasm runs only the tree-walker.
Options: (a) lower everything into the VM, delete the tree-walker, run the VM on wasm; the C++ diff (`cpl-conform diff`) is the second opinion. (b) keep the tree-walker as a slow reference oracle.
Recommendation: (a). Every semantic change is otherwise written three times (walker, VM, C++), and the playground runs different code from `cpl run`.
How to reverse it: the tree-walker is in git history; restore it behind a flag.
Meanwhile: keep both until the fallback count is 0 on corpus and conformance, then delete.

### lint-fmt: severity policy and the v0.3 lint list
Options: (a) `correctness` group (`float_equality` on `if a == b:` with floats, `memo_key_space`) is deny: `cpl lint` exits 1, `cpl build` still never fails on a lint; perf, idiom, style warn; pedantic allow. (b) Everything warns; deny only by `cp.toml [lints]`.
Scope options for v0.3: (1) the 38 lints as records plus L0601-L0609; (2) that plus the P0 and bound-aware families of the idiom catalog; (3) the whole catalog (~670 entries).
Recommendation: (a) and (2). Deny on correctness makes `cpl lint` a usable CI gate without ever blocking a submission; (2) is the set that prevents P0 bugs, and every lint still starts opt-in until its corpus run has 0 false positives (your earlier rule).
How to reverse it: levels are data in each lint record; a level change is one line per lint.
Meanwhile: every lint warns at its current level; work targets scope (2).

### lint-fmt: what `cpl fmt` may reformat, and where the line-1 stamp goes
Options: (a) one mechanical commit formats `corpus/problems` ports, `spec/std`, examples and docs snippets (about 550 files today); layout tests (`spec/conformance/trailing-comma.cp`, `bracket-layout.cp`, `space-indent-block.cp`, `crates/*/tests/fixes/*`) are exempt; the `// cpl 0.3.0` stamp goes on formatted files only, never on conformance cases or goldens (it shifts every line number). (b) Format on touch only; stamp everywhere.
Recommendation: (a). The corpus is the style reference agents learn from, so it should be fmt-clean; stamping test inputs churns every golden for no signal.
How to reverse it: the exempt list is one file in the fmt gate; the bulk commit is mechanical and revertable.
Meanwhile: nothing is bulk-formatted; new and touched files are formatted; no stamp yet.

### lint-fmt: design review of the v0.3 linter and formatter
The segment C review: plan/domains/lint-fmt/design/overview.md. The calls in it that shape the language: (1) the checker owns one fact table (ranges, proven indexes, sizes, trip counts) that lint, LSP and `cpl explain` all read, and the linter does no analysis of its own; (2) each lint is a data record in `spec/lints/<name>/` (before / after / near-miss) from which the Rust registry and the lint table in `spec/diagnostics.md` are generated; (3) the formatter stays token-preserving (formats broken files, reorders only through separately verified passes) rather than becoming a tree pretty-printer.
Recommendation: approve all three. (1) removes about 3k duplicated lines and the stdlib privilege; (2) makes the lint contract a test; (3) keeps 0 non-idempotent files and save-time formatting of broken code.
How to reverse it: (1) is reversible until `rules/bounds.rs` is deleted; (2) records can be compiled back into Rust consts; (3) a pretty-printer can replace `layout.rs` behind the same `format` API.
Meanwhile: M1 (fmt gate) and M2 (records, false-positive ratchet) proceed; they hold under every answer.

### idioms: are the principles and idiom catalog Ryan's normative taste?
Options: (a) `spec/idioms/principles.md` + `lints.md`, changes go through this queue, lint rules must cite a practice (e.g. practice 5 "absence is an optional": `print(best ?? -1)` over `-1` sentinels); (b) docs only (`docs/`), lint-fmt free to diverge.
Recommendation: (a); lint and the skill need one source of "idiomatic", and it is Ryan's taste.
How to reverse it: move the files to `docs/`, drop the cite rule.
Meanwhile: land them under `spec/idioms/` marked non-normative; lint cites them.

### idioms: the re-port bar for the 305 tracked ports
Options: (a) rethink each from `dream.cp` to the EXEMPLARS bar (slow, defines idiomatic); (b) migrate `port.cp` mechanically to v0.3 (fast, gives judge proof and the lint FP corpus, keeps C++ shape).
Recommendation: (b) first for all 305, then (a) in review-score order; only (a) counts toward the taste and token targets.
How to reverse it: stop migrating and go straight to (a); migrated ports stay as baselines.
Meanwhile: (b) then (a).

### idioms: what gates the skill
Options: (a) the 20 dry-run problems (known to skill writers, can overfit); (b) a fresh unseen 20 each release, written to the dry-run shape.
Recommendation: (b) as the gate, (a) as a regression check.
How to reverse it: gate on (a) only.
Meanwhile: report both; gate on (a) until the first fresh set exists.

### stdlib: Q1 what earns a prelude name
Today 192 names are in every program (`spec/std/prelude.cp`). Decision 150 (core R11.3.2) already dropped the 80-name budget: a name is in exactly when a `lib/` chapter's main tables list it, and the cost is a collision, not a count; `scripts/check.sh` still warns above 80. Usage (`python3 plan/domains/stdlib/measure.py prelude-usage`, table in `plan/domains/stdlib/prelude-usage.md`): of 305 reviewed ports and 100 hill-climb solutions, 5 names are used by at least 5% (`Mod`, `sum`, `Graph`, `Rooted`, `Comb`), 115 by at least one, 77 by none. The ports are v0.2 `port.cp` files, so they undercount v0.3-only names.
- (a) Keep decision 150: membership by the chapter tables; retire the 80 warning; use the usage table to review what a chapter's main table lists.
- (b) Decision 150 plus a usage floor: a table name used by no port or hill-climb solution needs a `use` (77 names today).
- (c) Restore a fixed budget of 80 (decision 94), cut by judgment.
Recommendation: (a). A rule a contestant can guess from the docs beats a usage cut that moves with the corpus; 5% would leave 5 names, which says usage alone is the wrong test. `cpl fix` (lint L0208) adds the `use` for any name that later leaves.
How to reverse it: a name moved out goes back with one line in `spec/std/prelude.cp`.
Meanwhile: keep 192; nothing removed until answered.

### stdlib: Q2 an int range where a type argument names a domain
174 doc examples fail with E0204 "takes a domain by name, and `0..3` is a range", e.g. `spec/std/dsu.cp`: `let d = Dsu<0..3>(); d.components == 3`, `spec/std/beats.cp`: `BeatsSegTree<0..3>(fill: 2).all_sum == 6`.
- (a) Accept the range as an anonymous domain argument (like decision 159's inline `(V, V)` domain). One-liners stay one line.
- (b) Require a declared domain: `domain D = 0..3` then `Dsu<D>()`; rewrite the 174 examples.
Recommendation: (a). It is how the examples and ports are written, and a named domain still works where the index space matters.
How to reverse it: make E0204 fire again; `cpl fix` can name the domain.
Meanwhile: examples stay as written and count as blocked; a compiler request is filed only after the answer.

### stdlib: Q3 where the compiler/std line is
`crates/cpl-ir/src/lower/call.rs` string-matches 89 builtin names (`ir_builtin_names`, the count agreed with tech-debt); 40 are public std names (`plan/domains/stdlib/measure.py privilege`), e.g. `sort`, `sorted`, `first`, `sum`, `prefix`. Same call as tech-debt Q1 (the builtin floor). A user library can't do what these do.
- (a) Intrinsics only for storage, I/O, bit operations and numeric primitives; everything else (`sort`, `min`, `sum`, `first`, ...) is a std body, moved only once its C++ matches the builtin (perf gate).
- (b) Keep a small blessed core vocabulary in the compiler, listed and frozen.
Recommendation: (a), the "no privileged stdlib" rule in AGENTS.md; the perf gate keeps it zero-cost.
How to reverse it: a moved name goes back into the builtin table; its std body stays as the reference.
Meanwhile: move nothing; std bodies become the reference the builtins are tested against, and the list is classified with tech-debt.

### debugger: what is the trace overhead bar measured against?
The 10^7-step budget test (`crates/cpl-interp/tests/trace_log.rs`) runs `var x = 0` / `for i in 0..10000000: x += i % 7`. Today: untraced 150 ms, observed-only 596 ms, recorded 996 ms.
Options: (a) recorded <= 3x untraced (today 6.65x scalar, 8.28x array; the hooks alone are 3.98x, so every VM speedup makes it harder); (b) recorded <= 3x the observed-only run (today 1.67x, passes); (c) an absolute budget: 10^7 recorded steps in <= 1 s, < 500 MB, step back < 50 ms, with the ratio to untraced reported but not gated.
Recommendation: (c). The contestant feels seconds, not ratios, and traced runs are on small shrunk inputs (the knapsack demo traces at 1.18x); a ratio to an ever-faster VM punishes VM wins.
How to reverse it: the assertion in `trace_log.rs` is one line; switch it back to the ratio.
Meanwhile: (c) is gated, (a) stays the M3 stretch target and is reported in `plan/domains/debugger/metrics.toml`.

### debugger: which commands make up the debugging surface?
`spec/tools/trace-debug.md` names `cpl trace d input`, `cpl snap` + `cpl run d --stage snap`, `cpl debug`. The CLI has `cpl run --trace knapsack.cp` (and `--live`), `cpl stress knapsack.cp --trace`, `cpl replay <file> --trace t.json --step n`, `cpl dap`.
Options: (a) the spec's commands, as aliases or renames; (b) today's: tracing is a flag on the command that ran (`run`, `stress`, `test`), `cpl replay` covers snapshots, `cpl dap` is the debugger; the spec is updated; (c) both.
Recommendation: (b). One flag on the commands a contestant already types is guessable, and `replay` of a recorded call already does what `snap` describes, without a second file format.
How to reverse it: add `cpl trace` / `cpl debug` as thin commands over the same code in `crates/cpl-cli`.
Meanwhile: (b); the lane asks the spec domain to update `spec/tools/trace-debug.md` and adds nothing new.

### debugger: where does the trace viewer live?
Today `tools/trace/index.html` is one self-contained 103 KB file (inline CSS and JS, no build), inlined by `crates/cpl-cli/src/trace.rs` (`include_str!`), loaded by `web/scripts/gen-debug.mjs` and the VS Code webview. `plan/DOMAINS.md` names a `web/packages/trace-viewer` that does not exist.
Options: (a) keep the one file in `tools/trace/`, add a browser test, and fix `plan/DOMAINS.md`; (b) a `web/packages/trace-viewer` TypeScript package with a build that emits the same single file for the CLI; (c) render traces in the site's React app and drop the offline file.
Recommendation: (a) for v0.3. Offline traces from `cpl run --trace` and VS Code need no build step, and 62 node tests already cover its state logic; (b) once the file passes about 3000 lines or needs shared components with the site.
How to reverse it: (b) is a move of `tools/trace/` plus a build step whose output the CLI includes.
Meanwhile: (a).

### spec: descending sort, a key only or also a `desc:` flag?
Chapter 7 (line 315) says "Descending is a key, not a flag: `by: x => desc(x.score)`", but `spec/std/seq.cp` gives `sort`/`sorted` a `desc: Bool = false`, and `spec/conformance/std-seq/sort-sorted-last.cp` uses it. Two spellings for one idea (rebuild round 2 hole H-SORT-DESC).
- A, key only: `copies.sorted(by: c => (c.start, desc(c.end)))` (corpus/problems/btoi18p4/spec.cp); plain descending is `xs.sorted(by: x => desc(x))`.
- B, both: `xs.sorted(desc: true)` for the whole order, `desc()` inside a key for one part.
Recommendation: A. One spelling, and `desc()` already covers mixed orders, which a flag cannot; 9 of the 100 hill-climb `spec.cp` files use `desc()`, none use the flag.
How to reverse: re-add the flag to the two signatures; no other spelling depends on it.
Meanwhile: chapter 7 wins; new cases use `desc()`; the existing case stays (never weakened) until this is answered, then stdlib drops the flag and the case is fixed with a codemod.

### spec: should `Subset<B>` and `Pair<V>` have `Int(p)` and a print form?
Both are domains (`[Subset<B>]T`, `[Pair<V>]T` work), but R04.3's `Int(p)`, neighbours and cuts are defined only on range-backed domains, and neither prints. Bitmask DP wants the mask: `for s in Subset<B>: dp[s] = ...` then `print(Int(s))` or `s + 1` is natural to try.
- A: `Int(s)` is the mask (`Subset<B>`) or the lower-triangle rank (`Pair<V>`), and both print as that integer form; no neighbours or cuts.
- B: also neighbours and cuts in walk order (`s.next`, `Range<Subset<B>>`).
- C: none; `s.mask` as a named member instead.
Recommendation: A. `Int(p)` already means "the position's integer form" for every other domain, so it is the guessable spelling; neighbours on subsets invite `s + 1` loops that skip the submask idioms.
How to reverse: remove the two `Int` members; the matrix marks the cells n/a again.
Meanwhile: the matrix (`spec/conformance/matrix.toml`, `[index.na]`) marks these cells n/a with the reason.

### spec: keep the `V.count` property beside the Seq method `V.count(f)`?
A domain is a `Seq` of its positions, so every Seq method applies (`V.find(f)`, `V.min(by: f)`), including `count(p)` from `spec/std/seq.cp`. But `V.count` with no call is already the number of positions (`domain V = read(1..=n)` then `[V]Int` has `V.count` slots).
- A, keep both: `let leaves = V.count(v => deg[v] == 1)` and `print(V.count)`.
- B, rename the property: `V.size` or `V.len`; `V.count` is only the Seq method.
- C, name the predicate: `V.count(where: f)`.
Recommendation: A. Arity tells them apart, both are the name a contestant guesses, and `V.count` is used across every chapter and the corpus; B is a corpus-wide rename for no bug class.
How to reverse: B with a `cpl fix` codemod.
Meanwhile: as ruled in R04.2.20 (on `devin/wip-conformance-matrix`).

### perf: which programs and which tail bar define the v0.3 runtime target?
Options: (a) the v0.3 perf set: dry runs with a matched `_ref.cpp`, `bench/<area>/*.cp`, corpus `spec.cp` ports, both sides >= 10 ms at max-N; p50 <= 1.1x, max <= 1.5x (AGENTS.md goal 3). (b) Same set, p95 <= 1.5x (what `plan/domains/perf/README.md` said). (c) The 0.2 343-port sweep in `bench/corpus-p95-report.md` (p95 2.61x today).
Recommendation: (a). The worst program is the one a contestant hits; (c) measures 0.2 ports being deleted.
How to reverse it: change the bar in `plan/domains/perf/metrics.toml`; the runner reports p95 and max either way.
Meanwhile: perf works to (a).

### perf: does Euclidean `%` on an unbounded `Int` count as abstraction cost?
`bench/zero-cost/optionals` is 1.01x against `direct.cp` but 1.26x against `hand.cpp` (`cpl bench --zero-cost --reps 11`); isolating it showed the gap is `cpl::emod(x * 31 + 7, m)` where `hand.cpp` uses C++'s truncating `%`, because `a[i]` is a plain `Int` that could be negative. `OptInt` itself is free.
Options: (a) the bench declares its input's real range (`0..10^9`), so the language proves `%` non-negative and prints plain `%`; (b) keep the source and count `emod` against the bar; (c) make codegen speculate (sign test, then fast path).
Recommendation: (a). Floored `%` is the language's semantics, not an abstraction; the range is the fact a contestant writes anyway.
How to reverse it: restore the `Int` declaration in `bench/zero-cost/optionals/abstract.cp`.
Meanwhile: perf reports optionals both ways and does not change the source.

### perf: to fit 32 KB, minify the C++ before dropping the CP Lang source from the header?
On main, fitting drops comments, then the source header, then half the indentation. `devin/wip-cpp-size-main` instead drops comments, indentation, needless spaces and numbered names first, and the source last, so the submission keeps what it was written from but its C++ reads worse.
Recommendation: minify first (the branch). The source is what a reader of the submission wants; the C++ is for the judge.
How to reverse it: the order is one list in `crates/cpl-codegen/src/lib.rs`.
Meanwhile: land the branch's order.

### infra: split the gate into a commit tier and a land tier?
Options: (a) keep one gate: pre-commit and `land.sh` both run `scripts/check.sh --fast`, 77 s after a one-line crate edit; (b) two tiers ([design](domains/infra/design/gate-tiers.md)): pre-commit runs the affected crates' tests (<= 10 s), `land.sh` and CI run today's full fast gate unchanged.
Recommendation: (b). Same tests guard main; a lane learns of a cross-crate break at land time instead of commit time.
How to reverse: point `.githooks/pre-commit` back at `check.sh --fast`.
Meanwhile: build (b) behind `CPL_GATE_TIER=commit`; the hook keeps running the full fast gate until it measures <= 10 s.

### infra: does red CI on main block landing?
Options: (a) warn only; (b) `land.sh` refuses while the latest main CI run is red, except commits marked `CPL_LAND_URGENT=1` (fixes and reverts).
Recommendation: (b) once CI has been green 24 h; today CI is red, so (b) would freeze every lane.
How to reverse: one env default in `scripts/land.sh`.
Meanwhile: (a), with the warning printed in every land.
Superseded (decisions.md 234): `land.sh` is rebase and push and checks no CI; CI gates every main tip and the infra sheriff reverts or fixes a red one.

### infra: what happens to unlanded devin/* branches?
Options: (a) delete only branches `git cherry` shows fully landed; (b) also archive to `refs/archive/<name>` and delete any branch with no owning lane after 48 h.
Recommendation: (b); 68 branches today make the queue hard to read, and archive refs keep the work restorable.
How to reverse: `git push origin refs/archive/<name>:refs/heads/<name>`.
Meanwhile: (a) only.

### tools: which commands does v0.3 ship (tools-1)
`spec/tools/cli.md` and `cpl --help` both list 23 commands, but 14 differ (`python3 plan/domains/tools/measure.py spec-sync`). Spec-only: `debug`, `fetch`, `interact`, `locate`, `migrate`, `selftest`, `trace`. Binary-only: `check`, `dap`, `fix`, `judge`, `replay`, `report`, `self`.
- A. Spec follows the binary: drop `fetch`, `locate`, `migrate` from v0.3.
- B. Binary follows the spec: add all 7, `cpl trace a in.txt` beside `cpl run a --trace`, `cpl debug a in.txt` beside `cpl dap`.
- C. Mixed: the spec adopts `check`, `fix`, `judge`, `self`, `report`, `replay`, `dap`; `trace` stays `cpl run a --trace` (one verb for running, as today), `debug` stays the editor's debugger over `cpl dap`, `interact` stays `cpl run a` on a file with an `@interactor` (`crates/cpl-cli/src/interact.rs`), `selftest` is dropped (it is `scripts/check.sh`); `fetch`, `locate`, `migrate` are built for v0.3.
Recommendation: C. One verb per job keeps the set guessable, and the three additions are real contest needs (samples, smallest failing input, library upgrades).
How to reverse it: the command table is data; adding an alias (`cpl trace` → `cpl run --trace`) or dropping a command is a one-line change plus its golden test.
Meanwhile: C is applied (decisions.md 244): `spec/tools/cli.md` lists the binary's commands; `fetch`, `locate`, `migrate` are the 3 left to build (`measure.py spec-sync`).

### tools: which editors are tier 1 (tools-2)
`editors/` has VS Code, Neovim, Vim, Emacs, JetBrains, Helix, Zed. On a lane VM only keyword parity runs; the rest skip for lack of binaries, and JetBrains fails to fetch Gradle artifacts (`CPL=target/release/cpl editors/smoke.sh`).
- A. All seven tier 1: every one in the gate (needs nvim, emacs, hx, Zed's wasm target and a JVM with network in the gate image).
- B. VS Code and Neovim tier 1 (installed in one step, smoke-tested in the gate, broken = main is red); the rest tier 2 (smoke when installed, JetBrains in CI, fixed on report).
- C. VS Code only.
Recommendation: B. Covers most contestants, keeps the gate fast and offline; tier 2 still shares the one `cpl lsp`.
How to reverse it: promote an editor by adding its smoke test to `scripts/check.sh`.
Meanwhile: B.

### tools: what `cpl explain` and hover show in v0.3 (tools-3)
`spec/tools/explain.md` lists representation sizes, niches, storage, padding, memory, specializations, checks, contexts, generic instances and output bytes. Today `cpl explain corpus/problems/cf2268b/port.cp` prints only closure lowering (`[static: direct] fn m1_sum_Lst_Int_Int_Int_93 param of -> cpl_fn0`).
- A. The whole spec for v0.3.
- B. The four sections contestants ask about: types with bytes and niches, arrays and peak memory, specializations chosen, checks left per build; in both `cpl explain` and hover, from one shared-core query. Contexts, generic instances, unsafe-tier sites after v0.3.
- C. Ship as is and fix the spec to match.
Recommendation: B. It answers "why is this slow/big" and keeps one source of truth; A depends on queries the compiler does not have yet.
How to reverse it: sections are independent records in `explain --json`; adding one later changes no other.
Meanwhile: B is applied as far as the data exists: `cpl explain <file>` lists the four sections under `Not shown yet:` until the compiler query lands.

### tools: which exit codes does `cpl` use (tools-4)
`spec/tools/cli.md` says 0 success, 1 program or test failure, 2 compile error, 3 usage error. The binary (`crates/cpl-cli/src/lib.rs`, `EXIT_ERRORS`/`EXIT_USAGE`/`EXIT_FAILED`) uses 1 compile error, 2 usage error, 3 the program failed: `cpl run` on `print(xs[5])` exits 3 (`run_exits_3_where_the_program_fails_and_keeps_what_it_printed`), `cpl judges a b` exits 2, the emitted C++ exits 3 on a failed `assert`.
Option A: change the spec to the binary. 2 for usage matches POSIX tools, getopt and clap; editors, `cpl test`, `cpl stress` and the emitted C++ already use these codes.
Option B: renumber the binary and the emitted C++ to the spec.
Recommendation: A. It costs one spec sentence. B changes three constants plus every script and editor that reads them, and gains nothing.
How to reverse it: swap the three constants in `crates/cpl-cli/src/lib.rs` and the `exit(3)` in the C++ runtime; the tests name the constants, not numbers (`crates/cpl-cli/tests/golden.rs`).
Meanwhile: A is applied (decisions.md 245): `spec/tools/cli.md` states the binary's codes.

### spec: what are a range's keys, its own elements or positions from 0?
Chapter 7 lists ranges as `Indexed<D, T>` (R07.3.5: `s[i]` in O(1), `s.keys: Range<D>`), but chapter 4's range methods (R04.4.12-21) never say what `r[i]`, `r.keys` or `r.items` mean. Today `let r = 2..5` gives `r[3]` = R0101 "index 3 is outside a range of length 3" (so `r[0]` is 2), and `r.keys`, `r.items` do not compile.
Option A, identity: `r[x]` is `x` for `x in r` (traps outside), `r.keys` is `r`, `for (k, x) in r.items` walks `(2, 2) (3, 3) (4, 4)`. A range is `V.range[l..r]` without the source: the same keys a slice keeps (R07.2.14), so `Range<V>` stays `Indexed<V, V>` and never mixes positions of `V` with counts.
Option B, positions: `r[0]` is 2, `r.keys` is `0..r.len`, items walk `(0, 2) (1, 3) (2, 4)`, as Python's `range(2, 5)[0]`. Familiar, but a `Range<V>` keyed by `Int` is the wrong-index-space bug the language exists to stop.
Option C: drop ranges from the `Indexed` row (they stay `Seq` + `Sized` + `Bidir` + `Contains`); `r[i]` is E0305.
Recommendation: A, or C if indexing a range has no use; B reintroduces the position-versus-value confusion.
How to reverse it: the three cases are generated from `spec/conformance/traits.toml`; change their expected output, or drop the `Range` claim of `Indexed`/`Keyed` there and in R07.3.5.
Meanwhile: the three `Range` lines stay in `spec/conformance/trait-gaps.txt`; no case asserts either reading.

### perf: does the judge build trap on a bad index, `/ 0` and `assert`?
Decision 240 (spec) says release has no checks unless kept: `xs[j]` with an unproven `j` is a raw load, `7 / d` with `d == 0` gives 0, `assert` is skipped (cases `spec/conformance/core-09/release-list-index-unchecked.cp`, `release-division-by-zero-fallback.cp`). AGENTS.md goal 5 says the interpreter and the C++ agree "including on input they reject", and the Trap Parity Gate (`crates/cpl-cli/tests/trap_parity.rs`) requires the `-O2`, no `-DLOCAL` judge build to stop on all 36 sites, among them `xs[three]` (site 1), `three / z` (13) and `assert z == 1` (25). Both cannot pass.
Option A (today): the judge build keeps bounds, division and assert traps; range stores and overflow are checked only under `-DLOCAL`. Cost is a compare per unproven index; ahead-of-loop checks (decisions 242 and on) remove most in loops. Measured remaining cost where they stay: `math/comb` emitted C++ 98 ms -> 81-86 ms with the table checks removed by hand, matched C++ 72-75 ms; `containers/min_heap` 121 ms -> 108 ms, `std::priority_queue` 110 ms (worklog, 2026-11-22).
Option B (decision 240): release drops them; a wrong index is C++ undefined behaviour as in hand-written C++, the gate's index, division and assert sites move to `-DLOCAL` only.
Recommendation: A. A wrong answer the judge reports as RE with a message we wrote is easier to explain than a silent WA, the cost left is in three stdlib types that can use `unchecked:` (requests filed with stdlib), and B gives up goal 5's promise.
How to reverse it: guard the tests in `cpl::at`, `ck` and the division helpers in `crates/cpl-codegen/src/runtime.rs` with `#ifdef LOCAL` (as `check_range` already is) and move the gate's sites to `known_traps.cp`.
Meanwhile: A; codegen keeps release traps, the two conformance cases stay failing, and the gate stays as it is.

## Answered
