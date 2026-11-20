The standard library is meant to feel like a good contest library, not a list of compiler exceptions. The language provides bounded indices, traits, generic code, lexical borrows, and a small intrinsic layer. Most containers and algorithms should be ordinary CP Lang you could write yourself.

> **Status.** This is a guide to the library design and selected prototype facilities, not a generated API reference for a released standard library. Exact module paths and signatures remain in progress. Follow the linked ports for implementations that actually run today.

## Sequences and collections

`[V]T` has a fixed index space; `List<T>` grows. `Map` and `Set` are ordered collections; `HashMap` and `HashSet` make hashing explicit. `Heap` takes an ordering instead of assuming every priority queue is a max-heap.

`[D]Bool` is an ordinary byte-per-element array. The approved packed alternative is an explicit, drop-in `Bitset<D>`, not a silent `vector<bool>`-style specialization with proxy-reference surprises. Choose it when packed storage or word-level operations are what you want.

Sequences share tools such as `pairs`, `windows`, `runs`, `scan`, and `rev`. Use a helper when it makes the actual algorithm clearer; there is no requirement to wrap a straightforward loop in a clever pipeline.

The [Modern Art port](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ccc21s2.cp) uses domain-indexed Boolean arrays for row and column parity. The [Meet-in-the-middle port](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/noi01p5.cp) uses a `HashMap<Int, Int>` to count sums.

## Range data structures

The planned core set is Fenwick trees, segment trees with lazy variants, sparse tables, and DSU with a rollback variant. Operation values choose behavior explicitly. A maximum segment tree and a sum segment tree share a data structure, not an implicit compiler guess about the operation.

```cp Library API shape · design fragment
use std.segtree.*

var best = SegTree<V, Int>(Max<Int>)
best.update(v, score)
let answer = best.fold(window)
```

This fragment assumes `V`, `v`, `score`, and `window` have been defined. Specialization should make the operation as direct as hand-written code. That is a performance target to verify, not a benchmark result claimed by this page.

## Graphs and trees

Graph APIs carry vertex and edge index types separately. Shortest-path results can use `[V]?Int` so an unreachable vertex is `none`, not a magic distance. The plan includes BFS, DFS orders, Dijkstra, topological sort, SCC, LCA, and Euler tours.

Typed handles from `Pool<Node>` support identity in trees and tries without storing borrows. A persistent data structure holds handles to owned storage; it does not smuggle an escaping slice through the borrow checker.

The [Minimum-spanning-tree port](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ccc23s4.cp) and [Rivers tree DP](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ioi05p6.cp) are useful concrete reading alongside the catalogue.

## Strings and mathematics

Strings reuse array indexing and slices. The library plan covers hashing, prefix and Z functions, and suffix arrays without changing the ASCII-only text model.

For modular arithmetic, the approved design separates `Mod<P>` for a compile-time modulus from `ModR` for a runtime modulus. Neither is a new arithmetic rule hidden inside `Int`. Other planned essentials include `gcd`, `lcm`, sieve utilities, factorial tables, binomials, and explicit `I128` helpers.

## Write a local library

A module is a `.cp` file. A library is a folder with `lib.toml`; project configuration maps a short import name to its location.

```text cp.toml · planned library configuration
[libs]
lib = { path = "../contest-lib" }
```

```cp CP Lang · design fragment
use lib.graph.flow as flow
use lib.treap.{Treap}
```

Export declarations with `pub`, and document them with `///` comments. Library modules contain declarations, not surprise input reads or mutable top-level state. The intended bundler follows imports, specializes generics, and includes only reachable code in the one-file C++17 submission.

Local libraries change during contests. The planned content-addressed store records exactly the sources used by a build, with explicit pinning and upgrades so old solutions remain reproducible. The `cpl test` and `cpl doc` workflows should work equally for your library and for `std`; neither is implemented in the current Rust CLI.

## Go to the source

Read the [standard-library plan](https://github.com/ryanbai1412/cp-lang/blob/main/plan/20-stdlib.md) for scope, [abstractions](https://github.com/ryanbai1412/cp-lang/blob/main/plan/13-abstraction.md) for traits and operation values, and the [prototype ports](https://github.com/ryanbai1412/cp-lang/tree/main/proto/ports) for full solutions. When older plan text disagrees with a later approved design, the approved `Index`, `lends`, ASCII strings, `Bitset<D>`, and `.l`/`.r` model is what this guide teaches.
