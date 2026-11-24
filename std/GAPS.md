# Library bodies the compiler can't express yet

One line per fn left `todo`, or written below its documented cost, because
of a compiler gap: module, fn, and what the compiler is missing. Remove a
line when the gap closes and the body lands.

- subset: every `Subset` method: `Subset` is built into the compiler (`crates/cpl-ir/src/lower/expr.rs`, `library_domain.rs`, cpl-check `prelude.rs`), so `cpl run` answers its calls itself instead of running these bodies.
- tree: `Lift.of(next)` without type arguments: `V` is not inferred from the lambda `next: V -> ?V` (E0302 "`Lift.of` needs a domain for `V`, and `Int` is not one"), so programs write `Lift<V, ()>.of(next)`.
- window, `undoing`/`Tracked` with a tuple-pattern closure passed straight to another call: `g.minimal(undoing(Dsu<V>(..), (mut d, (u, v)) => ..))` is E0302, because T must come from the outer call; a `let` binding takes T from a later `push` (std-window/undoing-infers-from-later-push). btoi20p3 writes `e: (V, V)`.
- io: `read_all`: CP Lang has no end-of-input test (the interpreter has one internally, `Input::at_eof`), so reading to the end can't be written; blocks ioi05p2 and the site's `read_all<Cmd>(until:)` snippets.
- seq, `filter` / `map`: `Filtered<S>` and `Mapped<S, U>` have no fields and no `T` parameter to hold the closure, and a struct cannot implement `Seq` yet; the compiler's built-in views serve these calls.
- bag, `SortedBag.split_at`/`join`, `SortedSet.split_at`/`join`: each bag owns its node pool, so O(log n) split/join between two bags needs a shared pool (or node moves) the struct layout does not have.
- segtree, `DynamicOrderTree.kth`: a Fenwick of order trees answers `count_lt` in O(log^2 |D|), but the k-th value over arbitrary online `T` needs a binary search over values on top of it (O(log^3 |D|)); O(log^2) needs the values known ahead (offline compression), which the signature doesn't take.
- lazy, `LazyTreap.split_at`/`append`: each treap owns its `nodes` list, so returning or absorbing another treap copies its nodes (O(n), not O(log n)); needs a shared node pool.
- seq, `compress`/`pieces`/`rebase`: a fn can now bind `domain C = 0..k` (as `with_domain` does), but a struct over it (`Compressed<C, T>`) keeps the callee's domain instead of unifying with the caller's `C` (E0302), and `(C:, Compressed(...))` misparses as a record field (E0103).
- seq, `compress`/`pieces`/`rebase`: each returns or keys by a fresh `domain` (`(domain C, ...)`, `t.Idx`), which a library fn cannot mint (same gap as `with_domain`).
- seq, `covering`: maps a range of cuts back to piece keys of `pieces`, which is itself blocked on fresh domains; nothing can build its `[P]Range<D>` input today.
- seq, `rev`, `Filtered.src`/`Mapped.src`/`Rev.src`/`Items.src`: the view structs are declared with no fields (`struct Rev<S>()`), so a body cannot hold or hand back its source; needs the views to carry `src`.
- seq, `drop`/`drop_last`/`take`/`take_last`/`split`/`maximal`/`minimal`/`Ring.maximal`/`Ring.minimal`: return `Slice<S>` views of a generic `S`, which cannot be built (`Slice<S>()` has no fields and `self[i..j]` on generic `S` indexes one element).
- segtree, `SegTree.new(over:)`: `SegTree<Int, Int>` is E0201 "doesn't know the name `D`" in `cpl run`, and the layout is one `bit_ceil(D.K)` array with no slot for nodes made on first touch.
- strings, `parse<T: Read>`: works as a compiler built-in; `Read` only reads standard input, so a std body needs `Read` over a token source (Q3).
- seq, `filter`/`map`/`items`, `Slice.keys`/`len`/`is_empty`/`iter`: lowered as compiler built-in views (`crates/cpl-ir/src/lower/call.rs`, `"map" | "filter"`, `"items"`); a std body needs the view structs to carry their source (as `rev` above) and a `Seq` impl per view that lowers as fast (Q3).
- derive: `Gen`, `gen`, `gen_fields` (README 4.17, not yet in `derive.cp`; `Hash` and `hash_fields` are): `Gen` is also the compiler's generator type (`crates/cpl-check/src/ty.rs`) and a `TRAIT_NAMES` entry in `crates/cpl-check/src/prelude.rs`, so a std trait of that name overlaps a compiler table. Held on Q3.
