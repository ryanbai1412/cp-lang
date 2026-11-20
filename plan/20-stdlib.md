# 2. Standard library

Status: decided unless a section says *proposed*. Folded from `dsa-catalogue.md`
(now in `archive/`); this copy is the source of truth.

## v1 scope

"doesn't need to get super fancy but should have the basics" (the user).
Every v1 module is written in the language on top of the intrinsics
(`Buffer<T>`, fast I/O, bit intrinsics, `@trusted` blocks), and the
abstractions in 1.4.

- **Containers:** `List`, `Deque`, `Heap`, `Map` (ordered), `HashMap`,
  `Set`, `HashSet`, `BitSet<D>`, `Pool<T>`.
- **Algorithms:** `sort` and `sorted` (with `by:`), binary search over
  predicates, `compress`, `next_permutation`, prefix sums via bounds.
- **Range structures:** `SegTree` (with an operation value and lazy
  variants), `Fenwick`, `SparseTable`, `Dsu` with a rollback variant.
- **Math:** `ModInt` (compile-time and runtime moduli), `gcd`, `lcm`,
  `pow_mod`, sieve, factorials and binomials, `I128` helpers.
- **Graphs and trees:** `Graph<V>`, `Tree<V>`, BFS, DFS orders, Dijkstra,
  topological sort, SCC, LCA, Euler tour.
- **Strings:** hashing, `z`, `prefix_function`, suffix array.
- **Views:** `runs`, `windows`, `pairs`, `scan`, `zip`, `enumerate`, `rev`,
  `chunks`.

Quality bar for every module:
- documented complexity and memory per element;
- laws tested with `cp fntest`;
- benchmarked within 5% of hand-written C++;
- tree-shaken to only what a submission uses.

The catalogue below lists everything beyond v1 that the abstractions must be
able to express.


Part 1 lists the data structures and algorithms that show up in contests.
Part 2 is how the language and stdlib would cover all of them with a small
set of abstractions. Design only; see `archive/operations.md` for operation and
ordering values in detail.

# Part 1. Catalogue

## Arrays and ranges
- Prefix sums (1-D, 2-D), difference arrays, 2-D difference
- Two pointers, sliding window (fixed and variable)
- Monotonic stack (next greater/smaller), monotonic deque (sliding max/min)
- Sparse table (RMQ), disjoint sparse table
- Fenwick tree (point update / prefix query, range update, 2-D)
- Segment tree (point update / range query, lazy propagation, beats,
  persistent, dynamic/sparse, merge, segment tree on indices / values)
- Sqrt decomposition, Mo's algorithm (with updates, on trees)
- Coordinate compression, discretization
- Sweep line with events
- Order statistics (k-th smallest, ordered set with rank)
- Inversion counting, merge sort tricks
- Binary search (on index, on answer, parallel binary search)
- Ternary search, golden-section search
- Divide and conquer (CDQ), offline query sorting

## Sets, maps, heaps
- Hash map / set (anti-hack hashing), ordered map / set
- Binary heap, heap with decrease-key, two-heap median
- Bitset (shift-or DP, transitive closure), bitmask subsets
- DSU (path compression + union by size, with data, weighted/potential,
  rollback, offline dynamic connectivity)
- Treap / implicit treap (split/merge, reversal), splay tree
- Trie (strings, binary trie for xor), persistent trie
- Linked lists (by index), skip list (rare)
- Li Chao tree, convex hull trick, kinetic segment tree

## Graphs
- Representations: adjacency list, edge list, adjacency matrix, CSR, implicit
  graphs (grids, states)
- BFS (0-1 BFS, multi-source), DFS (iterative), connected components
- Topological sort, DAG DP
- Shortest paths: Dijkstra, Bellman-Ford, SPFA, Floyd-Warshall, Johnson
- MST: Kruskal, Prim, Boruvka
- SCC (Tarjan, Kosaraju), condensation, 2-SAT
- Bridges, articulation points, biconnected / 2-edge-connected components,
  block-cut tree
- Euler path / circuit (Hierholzer)
- Flows: Dinic, push-relabel, min-cost flow, bipartite matching (Kuhn,
  Hopcroft-Karp), min cut, Hungarian
- Cycle detection, functional graphs, cycle finding
- Bipartiteness, graph coloring (small)
- Meet in the middle over states, A*, IDA* (rare)

## Trees
- Rooted tree from parent array / edges, DFS order (tin/tout), Euler tour
- LCA (binary lifting, Euler + RMQ), k-th ancestor
- Tree DP, rerooting DP
- Small-to-large merging, DSU on tree
- Heavy-light decomposition, centroid decomposition
- Virtual (auxiliary) tree
- Tree diameter, center, isomorphism hashing
- Link-cut tree (rare)

## Strings
- Polynomial hashing (rolling, double), substring compare
- KMP / prefix function, Z-function
- Aho-Corasick
- Suffix array + LCP, suffix automaton, suffix tree (rare)
- Manacher, palindromic tree (eertree)
- Trie on strings, string DP

## Math
- Modular arithmetic (ModInt), fast power, inverse, CRT
- Sieve (Eratosthenes, linear), factorization, divisors, Euler phi,
  Mobius
- GCD / extended GCD, linear diophantine
- Combinatorics: factorial tables, nCr, Catalan, Stirling, inclusion-
  exclusion
- Matrix multiplication / exponentiation (incl. min-plus)
- FFT / NTT, polynomial ops (inverse, log/exp: rare)
- Gaussian elimination (mod p, xor basis)
- Linear basis (xor), Berlekamp-Massey (rare)
- Miller-Rabin, Pollard rho
- Big integers (rare), fractions, floating point with eps
- Game theory: Sprague-Grundy, nim

## Dynamic programming
- Knapsack (0/1, unbounded, bounded), LIS (n log n), LCS, edit distance
- Interval DP, digit DP, bitmask DP (TSP, SOS DP), profile DP (broken
  profile)
- DP on trees, DAGs, grids
- Optimizations: prefix-min, monotonic queue, convex hull trick, Li Chao,
  divide and conquer, Knuth, aliens (Lagrangian), slope trick
- Rolling arrays, memoization, bottom-up tables

## Geometry
- Points / vectors, cross and dot products, orientation
- Segment intersection, point in polygon, polygon area
- Convex hull (monotone chain), rotating calipers
- Closest pair, half-plane intersection (rare), circle ops
- Sorting by angle, sweep with ordered set

## Patterns that aren't a single structure
- Multitest with per-test reset
- Interactive protocols
- Constructive output with a checker
- Randomization (hashing seeds, shuffles), stress testing
- Offline vs online processing, query batching
- Bit tricks (popcount, lowbit, subset enumeration)

# Part 2. Abstractions

The goal is a handful of core ideas that each cover a large part of Part 1.
Each idea is listed with what it is, what it covers and what's still open.

## A. Index spaces (domains)

What it is: a named, typed set of positions (`domain V = 1..=n`), with
arrays, sets and maps keyed by it.

Covers: arrays, prefix sums, every graph and tree algorithm (vertices and
edges are domains), grids (`[Row, Col]`, cells), coordinate compression
(`compress(xs)` returns a new domain), DP tables (`[I, J]?Int`), bitmask DP
(`Subset<D>`), DSU, heaps keyed by vertex.

Additions needed:
- **Domains derived from structure:** `tree.children(v)`, `Euler(tree)` (tin
  order as a domain), `Csr<V, E>` with typed edge ids.
- **Sub-domains:** `D.slice(l..=r)` and `Subset<D>` as domains, so a segment
  tree over a range is typed by it.
- **Dependent domains:** a separate index space per vertex (open question).

## B. Operation values

What it is: `Monoid`, `Group`, `Semiring`, `Action`, `Accumulator` as
`const` values with laws (see `archive/operations.md`).

Covers: prefix sums, Fenwick, segment trees (all variants), sparse tables,
sliding-window folds, Mo's algorithm, DSU with data, binary lifting, HLD,
rerooting, small-to-large, matrix exponentiation, min-plus DP, rolling
hashes, polynomial multiplication (a ring), xor basis (a vector space over
GF(2)).

## C. Ordering values

What it is: `Order<T>` (key-based by default), `Eq<T>`, `Hash<T>`,
`Monotone<T>`.

Covers: sorting, heaps, ordered sets and maps, sweep lines, binary search,
ternary search (`Unimodal<T>`), LIS, monotonic stacks and deques, Li Chao,
CHT, closest pair, angle sorting, Dijkstra.

## D. Handles and pools

What it is: `Pool<Node>` hands out typed handles; `?Node` is a 4-byte null.
No pointers.

Covers: treaps, splay trees, tries, persistent segment trees and tries,
dynamic segment trees, linked lists, suffix automata, Aho-Corasick, link-cut
trees, eertrees. Persistence is just "never overwrite, allocate a new node".

## E. Graph views

What it is: a graph is anything with `V`, `E` and `adj(v)`. Adjacency lists,
CSR, grids and implicit state graphs all provide the same view, so one
`bfs`, `dijkstra` or `scc` works on all of them.

```
let g = Graph<V>.from_edges(edges)           // CSR underneath
let dist = bfs(g, src)                       // [V]?Int
let grid = GridGraph(cells, open: c => grid[c] != '#')
let dist2 = bfs(grid, start)                 // [Cell]?Int
```

Covers: all of the Graphs section, plus trees as graphs with a root
(`Tree<V>.rooted(g, r)` gives `parent`, `depth`, `children`, `order`).

## F. Search and iteration patterns

What it is: generic algorithms over a predicate or callback instead of
copy-pasted loops:
`partition_point(lo..hi, pred)`, `two_pointers(...)`, `sweep(events, ...)`,
`submasks(m)`, `dfs` with enter/exit callbacks, generators (`yields T`) for
"produce in order, consume once".

Covers: binary search (all variants), two pointers, sliding windows, sweep
lines, subset enumeration, Euler tours, offline query processing.

Sequence views (stdlib generators that yield typed positions, compiled to
plain index loops):

```
for (s, x) in runs(a):                // maximal equal blocks; s is a Range1<V>
for (s, k) in runs(a, by: x => x > 0) // blocks with a constant key
for (u, v) in pairs(V)                // adjacent positions
for w in windows(V, k)                // length-k sub-domains; a[w] is a slice
for (k, grp) in groups(xs, by: e => e.c)
let pre = scan(a, Sum<Int>)           // running fold
```

`l` and `r` are positions in the array's domain, so `a[l..=r]`, `l.prev`
and `r.next` are bounds-safe and no `+ 1` / `- 1` appears.

## G. Memoization and DP tables

What it is: `@memo` with a named table, evaluation order and layout, plus
typed tables `[I, J]?T` with niche `none`.

Covers: all of the DP section. Optimizations are library fns that take the
cost as an operation (`knuth`, `dnc_opt`, `cht`, `aliens`).

## H. Numeric types

What it is: range types (`0..=1e9`), `Int` (64-bit), `ModInt<M>`
(runtime `M` allowed), `Frac`, `Float` with explicit `eps`, `BigInt`
(rare). Overflow is checked from ranges at compile time where possible.

Covers: the Math section. `ModInt` implements the `Field` operation value,
so matrix power, Gaussian elimination and NTT work over it unchanged.

## I. Program structure

What it is: `@multitest` with automatic per-test reset of owned state,
`protocol` / `@interactor`, `@checker`, `@brute`, `@invariant`, `read()`
specs with generators.

Covers: the "patterns" section.

# Coverage check

- **Arrays and ranges** — A, B, C, F
- **Sets, maps, heaps** — A, C, D
- **Graphs** — A, E, C (weights), B (path monoids)
- **Trees** — A, E, B, D (link-cut)
- **Strings** — B (hashing), D (automata, tries), A (positions)
- **Math** — H, B
- **DP** — G, A, B
- **Geometry** — H, C
- **Patterns** — I, F

# Gaps to resolve

1. **Dependent domains** (children of `v`, edges of `v`): today these fall
   back to lists and lose the index type.
2. **Segment tree beats and kinetic structures:** they need "break" and
   "tag" conditions on top of a monoid. Probably a separate `Beats` value.
3. **Flows:** they need a residual graph with paired reverse edges. A `Flow`
   graph type with typed edge pairs avoids the classic `e ^ 1` trick.
4. **Rollback (DSU with rollback, offline connectivity):** needs the
   `defer` / scoped-undo idea from earlier rounds.
5. **Geometry precision:** integer vs float geometry are different code
   paths; range types may pick `Int` when coordinates allow.
6. **Bitset DP:** `Bits<D>` with shifts. Needs word-level ops exposed
   safely.
7. **Persistence ergonomics:** keeping many versions (one root per update)
   must not copy. Handles make this cheap, but the API is still to design.
