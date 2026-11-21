# 26. Containers

**Layer:** library. Every type here is ordinary code over `alloc` and the `Index` trait; graphs are in `28`.

Status: [proposed].

## 26.1 Containers

| type | meaning |
|---|---|
| `List<T>` | growable array: `[D]T`'s (pointer, length) plus capacity |
| `Set<T>`, `Map<K, V>` | **ordered** by default (balanced tree). Navigation [p2 F7]: `s.first(after: x)` (>), `s.first(from: x)` (≥), `s.last(before: x)` (<), `s.last(upto: x)` (≤), `s.first`, `s.last`, `pop_first()`, `pop_last()`, all `?T` (maps: `?(K, V)`); sorted slices get the same family, defined by `partition_point` [r14 T14]; `remove(x) -> Bool`; `toggle(x) -> Bool` |
| `Counter<T>` | ordered multiset counts: `of(key)`, `add(key)`, `remove(key)`, `min()`, `max()`, `pop_min()` [r12 R2, p2 F7] |
| `RankedCounter<T>` | augmented tree, explicit by name: `kth(i, order:)`, `rank(x)` [p2 F7] |
| `HashSet<T>`, `HashMap<K, V>`, `HashCounter<T>` | hashing is opt-in by name [r7 11] |
| `Heap<T>(order:)` | binary heap; `order:` is required, because a wrong-direction heap is a silent WA [r11] |
| `Deque<T>` | ring buffer: `push_front`, `push_back`, `pop_front`, `pop_back` (`?T`), `front`, `back` [p2 F7] |
| `Dsu<V>`, `Dsu<V, M>(init:)` | union-find over a domain, `groups.root(v)`; with `M: CommutativeMonoid` (laws sampled by `cp test`), `groups.fold(v)` is the component's fold; `init:` as for `SegTree` [p1 F4, r13 P24] |
| `compress(xs) -> (domain H, Compressed<H, T>)` | `.value: [H]T`; `H(x)` traps when absent, `H.try(x) -> ?H` [p2 F4, r14 T15] |
| `Buckets<I, T>` | CSR lists when memory matters; `[I]List<T>` is the default [r8 11] |
| `Arena<T>` + handles | pointer-heavy structures [you] |

- Fallible lookup is always `.get(key) -> ?V`, and `m[key]` traps when absent. Counters use `counts.of(key)`
  (0 when absent) and `counts.add(key)`; the bracket error on a `Map<K, Int>` names `Counter` [r9 17, r11 N9, N31].
- Sizes are `.len` [r11 N9].
- `[D]T` is (pointer, length); `List<T>` adds capacity. `.list()` on a temporary `[D]T` allocates nothing new:
  it reuses the buffer and sets capacity = length [r10 31, r13 P5].

## 26.2 Round-3 additions [p3]

- **Ranges:** `r.contains(x)`.
- **`Dsu`:** `union -> Bool` (whether it merged); union by size with path halving.
