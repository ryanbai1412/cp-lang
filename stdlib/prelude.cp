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

use std.algebra.{Action, Desc, Monoid, desc, fold}
use std.bag.{SortedBag, SortedSet}
use std.bitset.{Bitset}
use std.deque.{Deque}
use std.dsu.{Dsu}
use std.fenwick.{Fenwick, Fenwick2D}
use std.flow.{Arc, FlowGraph}
use std.geometry.{ByAngle, Pt}
use std.graph.{Graph, Target, Tree, bfs, condense, dijkstra, peel, scc, topo}
use std.grid.{Dir4, Dir8, step}
use std.heap.{MaxHeap, MinHeap}
use std.interval.{IntervalMap}
use std.lazy.{AddMax, AddMin, AddSum, AssignMax, AssignMin, AssignSum, LazySegTree}
use std.marks.{Marks}
use std.math.{Sieve, bit, ceil_div, digits, fixed, gcd, lcm}
use std.mod.{Comb, Mod}
use std.prefix.{Diff, Prefix}
use std.segtree.{SegTree}
use std.seq.{
    bsearch_first,
    bsearch_last,
    compress,
    deduped,
    distinct,
    excl_fold,
    get_or_insert,
    insert,
    partition,
    remove,
    resized,
    runs,
    scan,
    windows,
    zip,
}
use std.sparse.{SparseTable}
use std.strings.{
    RollingHash,
    common_prefix_len,
    common_suffix_len,
    digit,
    ends_with,
    find,
    find_all,
    lower,
    repeat,
    starts_with,
    upper,
}
use std.tree.{Rooted}
