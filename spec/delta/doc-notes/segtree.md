# segtree doc notes

- `SegTree.fold`: contradiction. The stub declares `pub fn fold(self, r: Range<D>) -> T` twice (once after `apply`, once after `update`); both got the same contract. One should be deleted; left as is since names are not mine to change.
- `SegTree.clear`: contradiction. `spec/lib/ops-ranges.md` §7 says `s.clear()` goes "back to the build state"; the stub says "reset all leaves to `fill` (or `m.id`)". Followed the stub; chose that pending tags are dropped and an `of` tree keeps its formula.
- `SegTree.fold`, `try_fold`: the prose table says `fold` returns `?T` for a plain `Op`; the stub only has `Monoid` trees (`fold -> T`, `try_fold -> ?T`). Followed the stub.
- `SegTree.right_while`, `left_while`: chose a trap when `ok(identity)` is false (as ACL `max_right`/`min_left` assert), so the start cut always qualifies; the spec only says "`ok` monotone".
- `SegTree.get`: the stub says O(1); on a tree with `act:` pending tags must be applied, so documented O(log |D|) there.
- `SegTree.set`: chose that `set` replaces any tags applied to the position (the value read afterwards is exactly `x`).
- `SegTree.new`: chose traps for `a` together with `fill`, and `over:` together with `a`; with `over:` untouched positions read `fill` or the identity. Not stated.
- `SegTree.walk`: the stub says "canonical node aggregates"; chose the maximal aligned blocks of the `bit_ceil(|D|)` leaf tree (unique, so implementations agree).
- `SegTree.refresh`: chose a trap on a tree not built with `of`.
- `SegTree2D.new`: chose to require a `Commutative` op, since a rectangle fold has no single left-to-right order; the stub is silent.
- `MergeSortTree.count_in`, `WaveletMatrix.count_in`, `DynamicOrderTree.count_in`: chose 0 when `lo_v > hi_v`.
- `WaveletMatrix.kth`, `DynamicOrderTree.kth`: chose a trap on `k < 0` (`None` only for `k >= r.len`, as stated).
- `WaveletMatrix.rank`: chose 0 for `x` outside `0..V`, including negative `x`.
- `MergeSeg.merge`: chose that both inputs are consumed (nodes are shared); the stub's `self` is not `mut` and does not say.
- `MergeSeg.first_alive`: the stub's "lazily pruning dead leaves" makes removal permanent; documented that `alive` must be monotone over time.
- `MergeSortTree`, `SegTree2D`, `WaveletMatrix`, `DynamicOrderTree`, `MergeSeg`: only named in `spec/lib/ops-ranges.md` §8 ("stays named"); documented from the stubs.
