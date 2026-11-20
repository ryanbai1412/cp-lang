//! The prelude: the names of the standard library every program has
//! without writing a `use`. This file is the one list of them. `cpl
//! check`, `cpl run`, `cpl build`, `cpl lsp` and the playground all read
//! it, and bring in a name from it when a program uses that name and
//! neither declares nor imports its own.
//!
//! A name is added here as a `use` of the module that declares it, so
//! what it is and does is in that module's cp source, not in the
//! compiler. A library module says what it uses: only the program being
//! checked or run gets the prelude.

use std.algebra.{Action, Desc, Monoid, Reversible, desc, fold}
use std.bag.{SortedBag}
use std.bitset.{Bitset}
use std.deque.{Deque}
use std.dsu.{Dsu}
use std.fenwick.{Fenwick, Fenwick2D}
use std.flow.{Arc, FlowGraph}
use std.geometry.{ByAngle, Pt}
use std.graph.{Graph, Neighbours, Target, bfs, condense, dijkstra, peel, scc, topo}
use std.grid.{Dir4}
use std.heap.{MaxHeap, MinHeap}
use std.lazy.{AddMax, AddMin, AddSum, AssignMax, AssignMin, AssignSum, LazySegTree}
use std.marks.{Marks}
use std.math.{Sieve, bit, ceil_div, digits, fixed, gcd, lcm}
use std.mod.{Comb, Mod}
use std.prefix.{Prefix1, Prefix2, prefix1, prefix2, prefix_n}
use std.segtree.{SegTree}
use std.seq.{
    Compressed,
    bsearch_first,
    bsearch_last,
    compress,
    deduped,
    excl_fold,
    get_or_insert,
    insert,
    max_by,
    min_by,
    partition,
    remove,
    resized,
    runs,
    runs_by,
    scan,
}
use std.sparse.{SparseTable}
use std.strings.{HashVal, Hashable, RollingHash, ends_with, find, lower, repeat, starts_with, upper}
use std.tree.{Rooted}
