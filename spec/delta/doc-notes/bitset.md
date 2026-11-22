# Doc notes: `std.bitset`

- `Bitset.shift_up` / `shift_down` / `shift_or` (and the `DynBitset` ones):
  documented `k >= 0`; chose to trap on negative `k` (only `shifted` is signed).
- `Bitset.nth` / `DynBitset.nth`: chose to trap on `k < 0`; `None` stays for
  `k >= len`, as the stub says.
- `Bitset.subset_sums`: chose that position ordinal `s` (0-based from
  `D.first`) stands for the sum `s`, the empty sub-multiset makes 0 reachable,
  and negative weights trap. The stub says only "subset sums in `0..D.K`".
- `Bitset.each`, `first`, `any`: members visited in increasing order; `last`
  in decreasing order, so `pred` is called only up to the answer.
- `Bitset.len`: O(1), since `is_empty` says the count is kept current.
- `DynBitset`: chose to trap on indices outside `0..cap` for single-bit ops,
  on `cap < 0` / `new_cap < 0`, and on mismatched capacities in the word-wise
  ops (`union`, `intersect`, `subtract`, `symmetric`, `meets`); `next` and
  `count_below` also accept `i == cap`.
- `DynBitset.resize`: shrinking drops the bits at or above `new_cap`.
- Contradiction: `Bitset.prev(i)` is strictly before `i`, while
  `DynBitset.prev(i)` is `<= i`. Left both; documented the difference. (resolved: `Bitset.prev` counts `i`, decision 166)
- Contradiction: decision 62 says `Bitset` `empty`/`full` are `@get`; the stub
  declares `pub fn empty()` / `pub fn full()`. Left the stub. (resolved: decision 140 (later) wins: O(K / 64) constructors keep `()`)
- Contradiction: decision 66.1 adds `Bitset.find(pred)`; the stub has `first(pred)`
  and no `find`. (resolved: decision 66.1 wins: `first()` and `find(pred)`, decision 166)
- `XorTrie.new`: chose to trap unless `bits` is in `1..=62` (`2^bits` must fit a
  signed 64-bit `Int`).
- `XorTrie.add` / `xor_all` / queries: values outside `0..2^bits` trap;
  `remove`/`contains`/`count` of such a value just report absence.
- `XorTrie.xor_all`: the mask applies to elements present now; later `add`s
  store their value as given. `clear` also drops the mask.
- `XorTrie.count_xor_less`: `limit <= 0` gives 0, `limit >= 2^bits` gives `len`.
- `XorTrie.kth_xor`: ranks count multiplicity; `k < 0` traps.
- `XorTrie.mex_xor`: returns `2^bits` when every value is present.
