# Comparative inventory

Common operations from the C++ STL, Python builtins and `itertools`,
Rust's iterators and collections, and the AtCoder Library, each with its
CP Lang spelling (rule `std-api/R15`). The left-hand names are also the alias
table: searching for one finds the std name on the right
(handbook/domains/stdlib.md, "Findable without knowing the name").
`scripts/std-inventory.py` fails when a spelling names something
`cpl doc --format json std` doesn't document.

| From | Operation | CP Lang |
|---|---|---|
| C++ | `sort`, `stable_sort` | `a.sort()`, `sorted(a)` |
| C++ | `lower_bound` | `a.first_ge(x)` |
| C++ | `upper_bound` | `a.first_gt(x)` |
| C++ | `partition_point` | `a.bsearch(ok)` |
| C++ | `next_permutation` | `a.next_permutation()` |
| C++ | `reverse` | `a.reverse()`, the view `a.rev` |
| C++ | `accumulate` | `a.sum()`, `a.fold(op)` |
| C++ | `partial_sum` | `a.scan(+)`, `prefix_fold(+, a)` |
| C++ | `unique` | `a.dedup()` |
| C++ | `min_element`, `max_element` | `a.argmin()`, `a.argmax()` |
| C++ | `count` | `a.count(x)` |
| C++ | `nth_element` | `a.select(k)` |
| C++ | `rotate` | `rotate_left(a, k)` |
| C++ | `swap` | `swap(a, i, j)` |
| C++ | `gcd`, `lcm` | `gcd(a, b)`, `lcm(a, b)` |
| C++ | `priority_queue` | `Heap` |
| C++ | `set` | `SortedSet` |
| C++ | `multiset` | `SortedBag` |
| C++ | `map` | `SortedMap` |
| C++ | `unordered_map`, `unordered_set` | `HashMap`, `HashSet` |
| C++ | `deque` | `Deque` |
| C++ | `bitset` | `Bitset` |
| Python | `sorted` | `sorted(a)` |
| Python | `reversed` | `a.rev` |
| Python | `sum`, `min`, `max`, `any`, `all` | `a.sum()`, `a.min()`, `a.max()`, `a.any(p)`, `a.all(p)` |
| Python | `enumerate` | `a.items` |
| Python | `zip` | `zip(a, b)` |
| Python | `pow(a, b, m)` | `pow(a, b)` over `Mod` |
| Python | `math.comb` | `choose(n, k)` |
| Python | `math.isqrt` | `isqrt(n)` |
| Python | `collections.Counter` | `xs.tally()`, `Counter` |
| Python | `collections.deque` | `Deque` |
| Python | `heapq` | `Heap` |
| Python | `bisect_left`, `bisect_right` | `a.first_ge(x)`, `a.first_gt(x)` |
| Python | `itertools.accumulate` | `a.scan(+)` |
| Python | `itertools.groupby` | `a.runs()` |
| Python | `itertools.permutations` | `permutations(a)` |
| Python | `itertools.combinations` | `combinations(a, k)` |
| Python | `itertools.product` | `product(a, b)` |
| Python | `itertools.chain` | `chain(a, b)` |
| Python | `itertools.takewhile`, `dropwhile` | `a.take_while(p)`, `a.drop_while(p)` |
| Python | `random.shuffle` | `shuffle(a)` |
| Rust | `windows` | `a.windows(k)` |
| Rust | `chunks` | `a.chunks(k)` |
| Rust | `dedup` | `a.dedup()` |
| Rust | `binary_search`, `partition_point` | `a.bsearch(ok)` |
| Rust | `iter().rev()` | `a.rev` |
| Rust | `fold`, `scan` | `a.fold(op)`, `a.scan(op)` |
| Rust | `find` | `a.find(p)` |
| Rust | `flatten` | `a.flatten()` |
| Rust | `cycle` | `a.cycle()` |
| Rust | `BTreeMap`, `BTreeSet` | `SortedMap`, `SortedSet` |
| Rust | `HashMap`, `HashSet` | `HashMap`, `HashSet` |
| Rust | `VecDeque` | `Deque` |
| Rust | `BinaryHeap` | `Heap` |
| ACL | `dsu` | `Dsu` |
| ACL | `fenwick_tree` | `Fenwick` |
| ACL | `segtree` | `SegTree` |
| ACL | `lazy_segtree` | `SegTree(op, a, act: f)` |
| ACL | `suffix_array`, `lcp_array` | `suffix_array(s)`, `lcp(s, sa)` |
| ACL | `z_algorithm` | `z_function(s)` |
| ACL | `scc_graph` | `scc(g)` |
| ACL | `mf_graph` | `max_flow(g, s, t)` |
| ACL | `mcf_graph` | `min_cost_flow(g, s, t)` |
| ACL | `two_sat` | `TwoSat` |
| ACL | `modint` | `Mod` |
| ACL | `convolution` | `ntt(a, b)` |
| ACL | `floor_sum` | `floor_sum(n, m, a, b)` |
| ACL | `crt` | `crt(r, m)` |
| ACL | `inv_mod` | `x.inv` on `Mod` |
