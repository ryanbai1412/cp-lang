# A local library

What a contestant's own library looks like: a folder with a `lib.toml`,
one module per file, tests beside the code, and solutions that import
from it. It holds things the standard library leaves out, written on top
of it.

```text
examples/local-lib/
  lib.toml                    name = "lib"
  ds/                         data structures
  math/                       numbers, algebra, transforms
  dp/                         DP optimizations
  seq/                        folds over lists
  str/                        strings
  solutions/road_reversal.cp  a problem solved with the library
```

```text
$ python3 proto/cp.py test examples/local-lib
$ printf '3 2\n1 2\n2 3\n' | python3 proto/cp.py lib-run examples/local-lib/solutions/road_reversal.cp
YES
[0, 1, 0]
```

## What is in it, and why it is not in `std`

The standard library holds what most contests need most of the time. The
locked v2 design (`research/language-architecture-v2.md`, T13) moved
everything a problem needs once a season out of it and into a library like
this one, so the core stays small enough to know by heart. Each module
below is one file with its tests.

Data structures, in `ds/`:

- `rope.cp`: `Rope<T>`, a sequence with insert, erase, reverse and move of
  any stretch in O(log n) (an implicit treap).
- `range_kth.cp`: `RangeKth<D>`, the k-th smallest value and
  `count_below` on any range of a fixed array (a persistent segment tree).
- `li_chao.cp`: `LiChao` and `Line`, the lowest line at a point: the
  convex hull trick without sorted slopes or queries.
- `potential_dsu.cp`: `PotentialDsu<V>`, union-find that knows
  `pot(b) - pot(a)` inside a component and refuses a contradiction.
- `bipartite_dsu.cp`: `BipartiteDsu<V>`, union-find with a side per
  vertex: add edges, learn when the graph stops being two-colourable, and
  take edges back with `mark` / `undo_to`.
- `cover.cp`: `Cover<Y>`, the covered length under adding and removing
  intervals, for area-of-union sweeps.
- `two_stack_fold.cp`: `TwoStackFold<T>`, a queue that knows its fold for
  any monoid: the sliding window for `gcd`, `|` or matrices.
- `mo.cp`: `mo_order` and `steps`, Mo's algorithm as the visiting order
  plus the single moves between two windows.
- `xor_basis.cp`: `XorBasis`, a linear basis over GF(2).

Numbers, in `math/`:

- `frac.cp`: `Frac`, exact fractions that sort and round correctly.
- `hom.cp`: `Hom<T>`, affine maps under composition, and `BothWays<T>`,
  the `Reversible` value that lets `Rooted.fold_path` fold them along a
  tree path.
- `matrix.cp`: `Mat<P>`, square matrices over `Mod<P>`.
- `poly.cp`: `ntt`, `convolve` and `pow_trunc` modulo 998244353.
- `powers.cp`: `Powers<P>`, the powers of one base in O(1).
- `sos.cp`: `sos_sub`, `sos_sup` and `sos_sub_inv`, sums over subsets and
  supersets of bitmasks.

DP optimizations, in `dp/`:

- `dnc_opt.cp`: `monotone_minima` and `dnc_layer`, divide and conquer
  over a monotone best column.
- `knuth.cp`: `knuth`, interval DPs in O(n^2).

Sequences and strings, in `seq/` and `str/`:

- `fold_others.cp`: `fold_others` (for every position, the fold of all
  the others, with no inverse needed) and `fold_balanced`.
- `borders.cp`: `borders`, `period` and `overlaps`.
- `trie.cp`: `Trie<A>` over an alphabet that is a domain.
- `suffix_automaton.cp`: `SuffixAutomaton<A>`, built online.
- `hash2d.cp`: `Hash2D<R, C>`, the hash of any rectangle of a grid.

Two structures that used to be here are now in `std`, because the v2
design made them core: 2-SAT is `std.graph.TwoSat`, and union-find with
undo is `Dsu.mark()` / `Dsu.undo_to(snap)`.

## Importing

The name in `lib.toml` is the root of every import. A file anywhere below
the folder finds it by walking up, so a solution needs no configuration:

```
use lib.ds.bipartite_dsu.{BipartiteDsu}

let (m, k): (Int, Int) = read()
domain Road = 1..=m
var d = BipartiteDsu<Road>()
for _ in 0..k:
    let (a, b): (Int, Int) = read()
    d.add_edge(Road(a), Road(b))
if d.bipartite:
    print("YES")
    print([if d.find(r).1: 1 else: 0 for r in Road])
else:
    print("NO")
```

A library module imports the standard library the same way a solution
does, and takes what `std` takes. `two_stack_fold.cp` accepts a bare
operator because `Monoid<T>.of` does the work:

```
use std.algebra.{Monoid}

pub fn new<M>(op: M) -> TwoStackFold<T>:
    let m = Monoid<T>.of(op)
    TwoStackFold(m, [], m.id, [])

var q = TwoStackFold<Int>(gcd)
```

## The conventions it follows

Only `pub` items are visible to an importer; helpers stay private to
their file. A module holds declarations only, so importing it cannot run
code.

Structures are indexed by a domain. `PotentialDsu<V>` takes vertices,
`Trie<A>` takes letters of its alphabet, `Cover<Y>` takes ranges of its
cells. A structure whose positions shift with every edit (`Rope`) is the
exception, and says so: its positions are plain `Int`s.

Absence is `none`, not a sentinel. `LiChao.min_at` returns `?Int`
because a tree with no lines has no minimum; `RangeKth.kth` returns
`none` when the range is too short; `PotentialDsu.diff` returns `none`
for two vertices nothing relates.

A type with arithmetic implements the operator traits and then works
everywhere a number does. `Mat<P>` implements `Mul`, so `pow` is the
usual square-and-multiply, and `Mat<P>.product(n)` packages it as a
`Monoid` for a segment tree of matrices. `Frac` implements `Ord`, so
`sort`, `min` and `max` take it as they are.

A structure with no invariant between its cells may lend them.
`Mat<P>.at(i, j)` is a projection (`lends mut Mod<P>`), so
`m.at(i, j) += x` writes into the matrix and nothing is copied. The
structures that keep an invariant (a union-find, a Li Chao tree) expose
operations instead, so no caller can break them.

Every test checks against something simpler: `rope.cp` replays random
edits on a plain list, `range_kth.cp` sorts each range, `poly.cp`
multiplies the schoolbook way, `suffix_automaton.cp` counts substrings by
enumeration, `knuth.cp` runs the cubic DP. Most files end with one small
problem solved with the structure, which is the quickest way to see how
it is meant to be used.

## Starting your own

Copy the folder, keep `lib.toml`, and delete what you do not use. To use
a library that lives elsewhere, name it in the project's `cp.toml`:

```toml
[libs]
lib = "../my-cp-lib"
```
