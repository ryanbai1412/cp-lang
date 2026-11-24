# Library bodies the compiler can't express yet

One line per fn left `todo`, or written below its documented cost, because
of a compiler gap: module, fn, and what the compiler is missing. Remove a
line when the gap closes and the body lands.

- bag, `SortedBag.split_at`/`join`, `SortedSet.split_at`/`join`: each bag owns its node pool, so O(log n) split/join between two bags needs a shared pool (or node moves) the struct layout does not have.
- seq, `filter`/`map`/`items`: lowered as compiler built-in views (`crates/cpl-ir/src/lower/call.rs`, `"map" | "filter"`, `"items"`); the std view structs carry their source (`filter(a, p).src`), but the method forms `a.filter(p)`/`a.map(f)` are still lowered by the compiler (`a.filter(p).src` is E0203 on a `List`), so the std bodies run only in the free form; needs the built-in lowering deleted with a `Seq` impl per view that lowers as fast (Q3).
