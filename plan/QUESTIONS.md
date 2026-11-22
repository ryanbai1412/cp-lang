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
Options: (a) keep them as tabs with a "0.3 spec" badge and no Run, as today; the syntax and library tabs are all `checks` or `spec` (`web/content/landing/syntax/records.spec.cp`, `web/content/landing/library/ioi05p2.spec.cp`); (b) show only programs that run, adding the rest as the compiler catches up; (c) lead each tab group with runnable programs, spec ones after.
Recommendation: (c). The first thing a visitor clicks should run; the 0.3 shape still shows, labelled.
How to reverse it: tab order lives in `web/lib/landing.ts`; one edit.
Meanwhile: (c).

### site: should guide code blocks become live runners?
Options: (a) static highlighted blocks, as today (`web/content/docs/guides/tour.mdx`); (b) every block with status `runs` gets a compact Run and "Open in playground"; (c) only on Learn pages.
Recommendation: (b). The gate already proves which blocks run; Run turns reading into checking, which is the docs-only bar.
How to reverse it: one MDX component mapping in `web/components/`.
Meanwhile: (b), behind the same gate status as the landing.

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

## Answered
