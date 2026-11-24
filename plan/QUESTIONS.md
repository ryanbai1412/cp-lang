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
Today 192 names are in every program (`std/prelude.cp`). Decision 150 (core R11.3.2) already dropped the 80-name budget: a name is in exactly when a `lib/` chapter's main tables list it, and the cost is a collision, not a count; `scripts/check.sh` still warns above 80. Usage (`python3 plan/domains/stdlib/measure.py prelude-usage`, table in `plan/domains/stdlib/prelude-usage.md`): of 305 reviewed ports and 100 hill-climb solutions, 5 names are used by at least 5% (`Mod`, `sum`, `Graph`, `Rooted`, `Comb`), 115 by at least one, 77 by none. The ports are v0.2 `port.cp` files, so they undercount v0.3-only names.
- (a) Keep decision 150: membership by the chapter tables; retire the 80 warning; use the usage table to review what a chapter's main table lists.
- (b) Decision 150 plus a usage floor: a table name used by no port or hill-climb solution needs a `use` (77 names today).
- (c) Restore a fixed budget of 80 (decision 94), cut by judgment.
Recommendation: (a). A rule a contestant can guess from the docs beats a usage cut that moves with the corpus; 5% would leave 5 names, which says usage alone is the wrong test. `cpl fix` (lint L0208) adds the `use` for any name that later leaves.
How to reverse it: a name moved out goes back with one line in `std/prelude.cp`.
Meanwhile: keep 192; nothing removed until answered.

### stdlib: Q2 an int range where a type argument names a domain
174 doc examples fail with E0204 "takes a domain by name, and `0..3` is a range", e.g. `std/dsu.cp`: `let d = Dsu<0..3>(); d.components == 3`, `std/beats.cp`: `BeatsSegTree<0..3>(fill: 2).all_sum == 6`.
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

### stdlib: Q4 one `split` for strings and sequences
Answered 2026-11-23 by (a): decision 456, associated types chosen by impl.

### stdlib: Q5 decision 419's real searches need one name over two return types
Answered 2026-11-23 by (a): decision 456, associated types chosen by impl.

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


### tech-debt: which names may the compiler know (the builtin floor)?
Answered 2026-11-22 by the recommendation (Ryan's reversible-calls rule): decision 266, floor = storage, I/O, bits, numeric primitives.

### tech-debt: may the steward land pure-move refactors in other lanes' crates?
Answered 2026-11-22 by the recommendation: decision 267, yes, behaviour-free moves with a note in the owner's worklog.

### tech-debt: how strict are the debt ratchets?
Answered 2026-11-22 by the recommendation: decision 268, hard-fail; a named `--update` plus a decision raises one.

### stdlib: Q6 folds of an identity-less op are `?T` only through the op's type (needs compiler)
Answered 2026-11-23 by (a): decision 456, associated types chosen by impl.
