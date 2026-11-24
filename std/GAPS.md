# Library bodies the compiler can't express yet

One line per fn left `todo`, or written below its documented cost, because
of a compiler gap: module, fn, and what the compiler is missing. Remove a
line when the gap closes and the body lands.

- subset: every `Subset` method: `Subset` is built into the compiler (`crates/cpl-ir/src/lower/expr.rs`, `library_domain.rs`, cpl-check `prelude.rs`), so `cpl run` answers its calls itself instead of running these bodies.
- tree: `Lift.of(next)` without type arguments: `V` is not inferred from the lambda `next: V -> ?V` (E0302 "`Lift.of` needs a domain for `V`, and `Int` is not one"), so programs write `Lift<V, ()>.of(next)`.
- bag, `SortedBag.split_at`/`join`, `SortedSet.split_at`/`join`: each bag owns its node pool, so O(log n) split/join between two bags needs a shared pool (or node moves) the struct layout does not have.
- segtree, `DynamicOrderTree.kth`: a Fenwick of order trees answers `count_lt` in O(log^2 |D|), but the k-th value over arbitrary online `T` needs a binary search over values on top of it (O(log^3 |D|)); O(log^2) needs the values known ahead (offline compression), which the signature doesn't take.
- seq, `filter`/`map`/`items`: lowered as compiler built-in views (`crates/cpl-ir/src/lower/call.rs`, `"map" | "filter"`, `"items"`); the std view structs carry their source (`filter(a, p).src`), but the method forms `a.filter(p)`/`a.map(f)` are still lowered by the compiler (`a.filter(p).src` is E0203 on a `List`), so the std bodies run only in the free form; needs the built-in lowering deleted with a `Seq` impl per view that lowers as fast (Q3).
- derive: `Gen`, `gen`, `gen_fields` (README 4.17, not yet in `derive.cp`; `Hash` and `hash_fields` are): `Gen` is also the compiler's generator type (`crates/cpl-check/src/ty.rs`) and a `TRAIT_NAMES` entry in `crates/cpl-check/src/prelude.rs`, so a std trait of that name overlaps a compiler table. Held on Q3.
