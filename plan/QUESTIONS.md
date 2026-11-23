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

### idioms: are the principles and idiom catalog Ryan's normative taste?
Options: (a) `spec/idioms/principles.md` + `lints.md`, changes go through this queue, lint rules must cite a practice (e.g. practice 5 "absence is an optional": `print(best ?? -1)` over `-1` sentinels); (b) docs only (`docs/`), lint-fmt free to diverge.
Recommendation: (a); lint and the skill need one source of "idiomatic", and it is Ryan's taste.
How to reverse it: move the files to `docs/`, drop the cite rule.
Meanwhile: land them under `spec/idioms/` marked non-normative; lint cites them.

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
