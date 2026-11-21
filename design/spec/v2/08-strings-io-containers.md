# Spec chapter 8: strings, I/O and containers

Status: **draft v2** [proposed]. Fills review round 10 items 36, 37, 41, 42.

## 8.1 Strings

- `Str` is bytes. **It is indexed by the integer range `0..s.len`** [r7 12], so positions are plain
  integers. `s[i .. j]` is a slice and prints as text.
- `s[i]` is a `Byte`, with literals `'a'`. The stdlib's `Letter = labels('a'..='z')` converts with `Letter(c)`
  (checked) or `Letter.try(c) -> ?Letter`, like any conversion [r12 R4, p1 S1], so `[Letter]Int` replaces `[26]Int` plus `c - 'a'` [r11 N30].
- A read type may bound the length: `Str(len: 1..=1e4)`, checked like any range. A dense `@memo` over an
  unbounded `Str` length is a compile error naming this fix [r11 N27].
- `s.prefix_function() -> [s.len]0..s.len`, `s.z_function()`, `s.borders()` (proper non-empty border lengths,
  longest first) [r16 R7].
- Derived index spaces stay nominal: `SuffixArray(s)` returns `order: [Rank]0..s.len` and
  `lcp: [Rank]0..=s.len` over a fresh `Rank`, so a rank can't index the string.
- `s.digits()` gives `[s.len]0..=9`; `s.windows(m)` gives the nonempty length-m slices.
- Hashing: `Hash2` is `(Mint(mod: P1), Mint(mod: P2))` with a random base chosen on the judge, seed printed
  to stderr unless the judge profile counts stderr [r11 N34]. Local runs use a fixed seed; stress treats the seed as a generator input; `with hash(seed: s)`
  pins it [r7 13].

## 8.2 Reading

- `read<T>()` (or `let x: T = read()`) parses by type: integers, ranges types (checked), `Str`, tuples,
  `[D]T` (D.count items), nested arrays row-major, `[len: 1..=n]T` (length then items), structs field by
  field, `Perm` (bijectivity checked), `Graph<V>` / `[k]Edge<V>`.
- Under `with io(base: 1)`, counted positions and ranges of them convert; sizes, cuts, labeled domains and
  integer ranges don't (chapter 1).
- A `Range` reads as `first last`, inclusive, and traps unless `first ≤ last`, so it is nonempty.
- `domain V = read(1..=n)` reads a size into a domain.
- `read_line() -> Str` reads the rest of the current line [r11 N29].

## 8.3 Printing

- `print(x, y)` separates by the `io` context's `sep` (default space) and ends the line. `print(xs)` prints
  an array on one line; `print(xs, sep: "\n")` overrides.
- Bare `print(Bool)` is a compile error naming `yes(b)` [you]. `yes(b)` uses `io.answers`, default from the
  judge profile; `yes(b, answers: ("Possible", "Impossible"))` overrides locally.
- String interpolation: `"? {v}"`; positions interpolate in the file's base, and arrays use `io.sep`
  [r11 N32].
- Floats use `io.precision`.

## 8.4 Containers

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
| `Graph<V>` | CSR adjacency; `.adj(v)` yields `V`, `.out(v)` yields `(V, W)` when weighted [r16 R12], `.edges` view, `.undirected(from:, to:, weight:)`; `Edge<V, W>`; `g.components() -> (domain C, Partition)` [p1 F4] |
| `compress(xs) -> (domain H, Compressed<H, T>)` | `.value: [H]T`; `H(x)` traps when absent, `H.try(x) -> ?H` [p2 F4, r14 T15] |
| `Buckets<I, T>` | CSR lists when memory matters; `[I]List<T>` is the default [r8 11] |
| `Arena<T>` + handles | pointer-heavy structures [you] |

- Fallible lookup is always `.get(key) -> ?V`, and `m[key]` traps when absent. Counters use `counts.of(key)`
  (0 when absent) and `counts.add(key)`; the bracket error on a `Map<K, Int>` names `Counter` [r9 17, r11 N9, N31].
- Sizes are `.len` [r11 N9].
- `[D]T` is (pointer, length); `List<T>` adds capacity. `.list()` on a temporary `[D]T` allocates nothing new:
  it reuses the buffer and sets capacity = length [r10 31, r13 P5].

## 8.4b Round-3 additions [p3]

- **Reading:**
  - `Bool` reads `0`/`1` (its labels).
  - `read<Byte>()` skips whitespace, then reads one byte.
  - `[D]T` with a byte-labeled `T` (`Letter`, `Byte`) reads one contiguous token of exactly `D.count` bytes.
  - `[V]List<V>` reads count-prefixed rows.
  - The integer-range read lint is named `lint(read_range_index: allow)`.
- **Strings:** `Str.filled(len, b)`, `Str(bytes)`, `Letter.upper`.
- **Ranges:** `r.contains(x)`.
- **`Dsu`:** `union -> Bool` (whether it merged); union by size with path halving.
- **Graphs and trees:**
  - `bfs(from: roots, next: f) -> Search<D>` over any domain [r16 R10–R11]: `.dist: [D]?(0..D.count)`, `.parent`,
    `.order`, `.path(to:)`, `.tree()`, `.root` with `roots: true`. Sources dequeue in `from:` order and each
    level keeps its parents' order. It replaces `g.bfs_dist`, `g.bfs_order` and `g.bfs_tree`; on a graph,
    `next: g.adj`.
  - Grids: `cell.on_border`; stdlib `Dir4 = ring(labels(N, E, S, W))` with `cell.step(d) -> ?Cell` and
    `a.dir_to(b) -> ?Dir4`.
  - `Tree(edges).rooted(at: roots)` (forests)
  - `g.cycle() -> ?List<V>`
  - `t.euler()`, with `pos: [V]Order` and `span: [V]Range<Order>`
  - one constructor family: `Graph<V>.undirected(…)` / `.directed(…)`, each taking `edges:`, `from:`/`to:`,
    or `adj:`; children from a parent array are `I.group(by: parent)`
  - `g.topo_order() -> ?[V.count]V` (`none` on a cycle)

## 8.5 Open questions

1. ~~Per-string domain?~~ Keep `0..s.len`; revisit if ports show cross-string index bugs [r11].
2. ~~Default `Heap` order?~~ None; `order:` required [r11].
