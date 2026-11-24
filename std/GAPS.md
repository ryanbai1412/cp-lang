# Library bodies the compiler can't express yet

One line per fn left `todo`, or written below its documented cost, because
of a compiler gap: module, fn, and what the compiler is missing. Remove a
line when the gap closes and the body lands.

- subset: every `Subset` method: `Subset` is built into the compiler (`crates/cpl-ir/src/lower/expr.rs`, `library_domain.rs`, cpl-check `prelude.rs`), so `cpl run` answers its calls itself instead of running these bodies.
- tree: `Lift.of(next)` without type arguments: `V` is not inferred from the lambda `next: V -> ?V` (E0302 "`Lift.of` needs a domain for `V`, and `Int` is not one"), so programs write `Lift<V, ()>.of(next)`.
- seq, `filter` / `map`: `Filtered<S>` and `Mapped<S, U>` have no fields and no `T` parameter to hold the closure, and a struct cannot implement `Seq` yet; the compiler's built-in views serve these calls.
- bag, `SortedBag.split_at`/`join`, `SortedSet.split_at`/`join`: each bag owns its node pool, so O(log n) split/join between two bags needs a shared pool (or node moves) the struct layout does not have.
- segtree, `DynamicOrderTree.kth`: a Fenwick of order trees answers `count_lt` in O(log^2 |D|), but the k-th value over arbitrary online `T` needs a binary search over values on top of it (O(log^3 |D|)); O(log^2) needs the values known ahead (offline compression), which the signature doesn't take.
- lazy, `LazyTreap.split_at`/`append`: each treap owns its `nodes` list, so returning or absorbing another treap copies its nodes (O(n), not O(log n)); needs a shared node pool.
- seq, `compress`/`pieces`: a fn can now bind `domain C = 0..k` (as `with_domain` does), but a struct over it (`Compressed<C, T>`) keeps the callee's domain instead of unifying with the caller's `C` (E0302), and `(C:, Compressed(...))` misparses as a record field (E0103).
- seq, `compress`/`pieces`: each returns a fresh `domain` (`(domain C, ...)`), which a library fn cannot mint (same gap as `with_domain`).
- seq, `covering`: maps a range of cuts back to piece keys of `pieces`, which is itself blocked on fresh domains; nothing can build its `[P]Range<D>` input today.
- seq, `drop`/`drop_last`/`take`/`take_last`/`split`/`maximal`/`minimal`/`Ring.maximal`/`Ring.minimal`: return `Slice<S>` views of a generic `S`, which cannot be built (`Slice<S>()` has no fields and `self[i..j]` on generic `S` indexes one element).
- segtree, `SegTree.new(over:)`: `SegTree<Int, Int>` is E0201 "doesn't know the name `D`" in `cpl run`, and the layout is one `bit_ceil(D.K)` array with no slot for nodes made on first touch.
- strings, `parse<T: Read>`: a compiler built-in that runs the same `Read` impls as `read<T>()` over the string, local `type Z = Mod(p)` included; `Read` only reads standard input, so a std body needs `Read` over a token source (Q3).
- seq, `filter`/`map`/`items`: lowered as compiler built-in views (`crates/cpl-ir/src/lower/call.rs`, `"map" | "filter"`, `"items"`); the std view structs carry their source (`filter(a, p).src`), but the method forms `a.filter(p)`/`a.map(f)` are still lowered by the compiler (`a.filter(p).src` is E0203 on a `List`), so the std bodies run only in the free form; needs the built-in lowering deleted with a `Seq` impl per view that lowers as fast (Q3).
- derive: `Gen`, `gen`, `gen_fields` (README 4.17, not yet in `derive.cp`; `Hash` and `hash_fields` are): `Gen` is also the compiler's generator type (`crates/cpl-check/src/ty.rs`) and a `TRAIT_NAMES` entry in `crates/cpl-check/src/prelude.rs`, so a std trait of that name overlaps a compiler table. Held on Q3.
