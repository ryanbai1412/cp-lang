# lazy doc notes

- `LazyTreap.len`, `LazyTreap.is_empty`: `@get` sat on its own line above `pub fn`, so the audit (and doc tools) did not see the doc; joined into `pub @get fn` as everywhere else. No signature change.
- `LazyTreap.new`: the meaning of `apply`'s `k` and of `split` is only sketched in the stub and decision 68; chose `k` = rank within the run the tag was applied to, with `split` responsible for shifting the right child's ranks (default `(f, f)` only for rank-free tags).
- `LazyTreap.insert`: chose "before the first `e` with `before(x, e)`, else at the end", so equal elements keep insertion order (the stub says "first position where `before(x, elem)` holds").
- `LazyTreap.apply_range`, `apply`, `reverse`, `erase_range`, `insert_at`, `erase_at`, `split_at`, `get`: bounds are unstated; chose traps outside `0..len` (`0..=len` for cut-like arguments), empty ranges change nothing.
- `LazyTreap.reverse`: chose that earlier tags keep their effect on the elements they reached and later tags see the new order; the stub does not say how reversal interacts with rank-dependent tags.
- `LazyTreap.take`: chose a trap on `k < 0`; the stub says only "first `min(k, len)`".
- `LazyTreap.append`: chose that `self`'s functions are kept and both treaps must agree; unchecked.
- `LazyTreap.reserve`: chose "no change when `n` is below capacity" and a trap past the `2^31` size bound (decision 122).
- `Rope.new`: the stub says "initialized from `a`, `fill`, or a tag `merge` operation"; chose `a` xor `fill` (trap otherwise) and `merge(tag, x)` as how `apply` combines a tag into a value, with overwrite when `merge` is absent. Nothing else in the spec defines `Rope`; it needs a design pass.
- `Rope.query_path`: the result depends on which tags are still pending, i.e. on the random tree shape, so two implementations can disagree; documented the constraint on `f` instead of a full rule. Contradiction with "two implementers build the same function"; flagged for redesign (e.g. drop it or define it over the latest covering tag).
- `Flip.op`, `flip_action`: identity and composition (exclusive or) are not stated; chose `Flip(id, id)` and xor, the only lawful choices.
