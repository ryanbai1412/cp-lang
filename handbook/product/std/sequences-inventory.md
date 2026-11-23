# Sequences: prior-art inventory

| | |
|---|---|
| Kind | Evidence for [sequences-usecases.md](sequences-usecases.md) |
| Owner | stdlib |

Every member of four libraries a contestant already knows maps to a
sequences use case (`seq/U<n>`), to another std surface, or to a non-goal. A
member with no home would be a missing use case; there are none left. "Other
surface" names the [std area](README.md) surface that owns it.

## C++ `<algorithm>` and `<numeric>`

- **Search:** `find`, `find_if`, `find_if_not` go to U1 and U2;
  `find_end`, `search`, `search_n` go to strings (pattern search);
  `find_first_of` and `adjacent_find` go to U2 over `keys`; `count` and
  `count_if` go to U18; `mismatch`, `equal` and `lexicographical_compare` go
  to U31; `all_of`, `any_of` and `none_of` go to U18.
- **Sorted search:** `lower_bound`, `upper_bound` and `binary_search` go to
  U6 and U27; `equal_range` goes to U28; `partition_point` goes to U27.
- **Extremes:** `min`, `max`, `min_element` and `max_element` go to U3 and
  U4; `minmax` and `minmax_element` go to U32; `clamp` goes to math.
- **Folds:** `accumulate` and `reduce` go to U9 and U36;
  `transform_reduce` and `inner_product` go to U9 over `zip`;
  `partial_sum`, `inclusive_scan` and `exclusive_scan` go to U7, U8 and
  U38; `adjacent_difference` goes to U14 (`adjacent`).
- **Sorting:** `sort` and `stable_sort` go to U12; `partial_sort` and
  `nth_element` go to U34; `is_sorted` and `is_sorted_until` go to U30;
  `partial_sort_copy` goes to U34 (`min_k`).
- **Sorted sets:** `merge` and `inplace_merge` go to U54; `set_union`,
  `set_intersection`, `set_difference`, `set_symmetric_difference` and
  `includes` go to U55; `unique` goes to U19 and U53.
- **Permutations:** `next_permutation`, `prev_permutation` and
  `is_permutation` go to U49 and U30.
- **Editing:** `copy`, `copy_if`, `copy_n` and `copy_backward` go to U17
  and U56; `move` is a non-goal (values move by default); `fill`,
  `fill_n`, `swap`, `swap_ranges`, `iter_swap`, `reverse`, `rotate` and
  `shift_left` go to U58; `remove` and `remove_if` go to U56 (`filter`);
  `replace` and `replace_if` go to U56 (`map`); `transform` goes to U56;
  `generate` and `iota` go to U65; `reverse_copy` and `rotate_copy` go to
  U16 and U58.
- **Partitions:** `partition`, `stable_partition`, `partition_copy` and
  `is_partitioned` go to U56 (`filter` twice) and U27.
- **Heaps:** `make_heap`, `push_heap`, `pop_heap` and `sort_heap` go to
  sorted containers (`Heap`).
- **Numeric:** `gcd`, `lcm` and `midpoint` go to math.
- **Non-goals:** `shuffle`, `sample` and `random_shuffle` live in stress
  generators (U64); the execution policies are out of scope.

## Python builtins and `itertools`

- **Builtins:** `len` and `sum` go to U18; `min` and `max` (with `key=` and
  `default=`) go to U4; `sorted` and `list.sort` go to U12; `reversed` goes
  to U16; `enumerate` goes to U15 (`items`); `zip` goes to U15; `map` and
  `filter` go to U56; `any` and `all` go to U18; `range` goes to U65 and
  U45; `str.join` goes to U24; `list.index` goes to U2;
  `list.count` goes to U18; `list.insert`, `pop`, `remove`, `append` and
  `extend` go to U58 and U21; `next(gen, default)` goes to U20 and U44.
- **`itertools`:** `accumulate` goes to U7 and U23; `chain` and
  `chain.from_iterable` go to U22; `compress` goes to U56; `dropwhile`,
  `takewhile` and `islice` go to U44 and U45; `groupby` goes to U10;
  `pairwise` goes to U14; `batched` goes to U14 (`chunks`); `starmap` goes
  to U56; `tee` goes to U63; `zip_longest` goes to U15; `product`,
  `permutations`, `combinations` and `combinations_with_replacement` go to
  U50 and U49; `count`, `cycle` and `repeat` go to U59.
- **`collections` and `bisect`:** `Counter` goes to U41; `deque` goes to
  U61; `bisect_left`, `bisect_right` and `insort` go to U6, U27 and sorted
  containers.

## Rust `Iterator` and slices

- **Search:** `find`, `rfind` and `find_map` go to U1 and U20; `position`
  and `rposition` go to U2; `any`, `all` and `count` go to U18.
- **Extremes:** `max`, `min`, `max_by_key` and `min_by_key` go to U4 and
  U3. Rust returns the last maximum and the first minimum; ours takes
  `ties:` (U3).
- **Folds:** `fold`, `try_fold` and `reduce` go to U9 and U36; `sum` and
  `product` go to U18; `scan` goes to U23.
- **Adapters:** `map`, `filter`, `filter_map`, `flat_map` and `flatten` go
  to U56 and U22; `take`, `skip`, `take_while`, `skip_while` and
  `map_while` go to U44; `step_by` goes to U45; `chain` goes to U22; `zip`,
  `enumerate` and `unzip` go to U15 and U57; `rev` goes to U16; `peekable`
  and `fuse` are non-goals (a `for` loop has the same power); `cycle` goes
  to U59; `partition` goes to U56; `inspect` goes to the debugger.
- **Slices:** `sort`, `sort_by_key`, `sort_unstable` and `sort_by_cached_key`
  go to U12; `select_nth_unstable` goes to U34; `binary_search`,
  `partition_point` go to U27; `windows`, `chunks`, `chunks_exact`,
  `rchunks` and `chunk_by` go to U14 and U10; `split`, `splitn` and
  `split_inclusive` go to U43; `dedup` and `dedup_by_key` go to U53;
  `reverse`, `rotate_left`, `rotate_right`, `swap`, `fill`, `copy_from_slice`
  go to U58; `concat` and `join` go to U22 and U24; `starts_with` and
  `ends_with` go to U31; `first`, `last` and `split_first` go to U21 and
  U60; `contains` goes to U18; `iter().rev().enumerate()` is the U16 trap.

## AtCoder Library

Nothing in ACL is a sequence call; each module maps to another surface.

- **Range structures:** `segtree`, `lazy_segtree` and `fenwick_tree`.
- **Sorted containers and graphs:** `dsu`, `scc_graph`, `two_sat`,
  `maxflow` and `mcf_graph` go to graphs.
- **Strings:** `suffix_array`, `lcp_array`, `z_algorithm`.
- **Math:** `modint`, `convolution`, `pow_mod`, `inv_mod`, `crt` and
  `floor_sum`.
