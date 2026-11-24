# Friction report: ioi25_festival (memory)

Festival's port used 1.22x the reference's peak RSS at `--max` (n = 2e5): 11.9 MB against
orig.cpp's 9.8 MB. massif (`valgrind --tool=massif`, g++-14 -O2 -g) names the heap sites at
the port's peak, inside `C.filter(...).sorted(by: ...)` for `mult`:

| Site in the generated C++ | Bytes | orig.cpp |
|---|---|---|
| `coupons`, `std::vector<std::tuple<ll, ll>>` (`read<[C](p: 1..=1e9, t: 1..=4)>()`) | 3.20 MB | two `vector<int>`, 1.60 MB |
| `cpl::sort_by` keys, `std::vector<m1_Ratio>` (16 B per key) | 2.42 MB | none: `std::sort` with a comparator |
| `cpl::sort_by` order, `std::vector<size_t>` | 1.21 MB | none |
| `stable_sort_lt` merge buffer over that order | 1.21 MB | none |
| the filtered list being sorted, `std::vector<int>` | 1.05 MB | its sorted pairs, 2.10 MB |

Heap peak 9.29 MB against 7.07 MB. A trivial program (read one value, print it) is 3.8 MB
against 3.7 MB, and the port's reader streams 64 KB at a time, so neither runtime nor input
buffering is the gap. The first row was in the port and is fixed there (below); the second to
fourth are `cpl::sort_by`. With the fix the port is at 10.3 MB against 9.8 MB (1.05x) and
0.04 s against 0.06 s.

## `read<T>()` stores a narrow table wide, and `cpl explain` says it doesn't
- **Where:** `port.cp` as first written, `let coupons = read<[C](p: 1..=1e9, t: 1..=4)>()`.
- **Repro:** the same program with each spelling, 2e5 random coupons, best of 3:
  ```cp
  let n: 1..=2e5 = read()
  domain C = 0..n
  let coupons = read<[C](p: 1..=1e9, t: 1..=4)>()     // or: let coupons: [C](p: 1..=1e9, t: 1..=4) = read()
  print(C.count(c => coupons[c].t > 1))
  ```
  `read<T>()` emits `std::vector<std::tuple<ll, ll>> coupons` (6.5 MB peak RSS); the
  annotated `let` emits `std::vector<std::tuple<uint32_t, uint8_t>>` (5.0 MB, as the C++ with
  two `vector<int>`). `cpl explain` reports `8 B per element x 200000 = 1.6 MB` for both.
- **Also:** a table whose length is a read value keeps `ll` elements in either spelling, with
  the same wrong `explain` line: `let n: 1..=10_000 = read()` then
  `let parents: [n - 1]0..n = read()` emits `std::vector<ll> parents` while `explain` says
  `2 B per element x 9999`. A `List<0..=4>` is a `std::vector<ll>` too. ioi25_migrations pays
  for both (its `parents` and the message list `s`).
- **What felt wrong:** the two spellings mean the same, and the cost report claims the narrow
  one for both. Nothing in `explain` or a lint points at the 2x.
- **Goal it hurts:** costs are visible, zero-cost safety.
- **Ideal code:** either spelling, stored as `explain` says.
- **General fix (codegen):** pick the element type from the table's element range wherever
  the type comes from (an annotation, `read<T>`, or a runtime length), and narrow `List<R>`
  elements the same way; then derive `explain`'s storage line from the emitted C++ type, so the
  two can't disagree.
- **Port fix:** the annotated `let`. 11.9 MB to 10.3 MB; the `--max` output is unchanged and
  `scripts/corpus.py verify ioi25_festival` stays clean.
- **Severity:** high (a silent 2x on the main input table, and the cost report is wrong).

## `sorted(by:)` allocates three n-sized buffers beside the list
- **Where:** `port.cp` and `dream.cp`,
  `C.filter(c => coupons[c].t > 1).sorted(by: c => Ratio(coupons[c].p * coupons[c].t, coupons[c].t - 1))`.
- **Repro:** 2e5 random coupons, best of 3; the C++ is `std::stable_sort` with a comparator:
  ```cp
  let n: 1..=2e5 = read()
  domain C = 0..n
  let coupons: [C](p: 1..=1e9, t: 1..=4) = read()
  let mult = C.filter(c => coupons[c].t > 1).sorted(by: c => coupons[c].p * coupons[c].t)
  print(mult[0])
  ```
  9.2 MB peak RSS against 6.2 MB (1.49x); heap 6.45 MB against 3.25 MB. `cpl::sort_by`
  (crates/cpl-codegen/src/runtime/lists.rs) computes every key into a `vector<K>`, sorts a
  `vector<size_t>` of indices with `stable_sort_lt` (which allocates its own buffer), then
  moves the elements through a third `vector<T>`.
- **What felt wrong:** `sorted(by: key)` is the obvious way to sort, and it costs
  n * (sizeof K + 16 + sizeof T) bytes on top of the list.
- **Goal it hurts:** performance (memory), the obvious way is the fast way.
- **Ideal code:** the line above at the cost of `std::stable_sort` with a comparator.
- **General fix (codegen):** when the key is cheap and pure (field reads and arithmetic, as
  here), sort the list in place with a comparator that recomputes the key; keep the cached
  keys for expensive keys, and then sort (key, element) pairs in one buffer, not keys plus
  an index permutation plus a copy.
- **Severity:** medium (1.5x peak RSS on an n log n sort; 2.4 + 1.2 + 1.2 MB in festival).
