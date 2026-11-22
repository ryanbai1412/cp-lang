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

use std.algebra.{Action, Commutative, Desc, Group, Idempotent, Monoid, Op, Ordering, Selective, Semiring, add, affine, assign, cmp, desc, each, flip, fold}
use std.beats.{BeatsSegTree}
use std.bag.{BucketCount, SortedBag, SortedMap, SortedSet}
use std.bitset.{Bitset}
use std.deque.{Deque, LinkedList}
use std.dp.{LiChao, Line, LineContainer, MonotoneHull, aliens}
use std.dsu.{Dsu, DsuOf, ParityDsu, PotentialDsu}
use std.fenwick.{Fenwick, Fenwick2D}
use std.flow.{Arc, FlowGraph}
use std.geometry.{ByAngle, Location, Pt, locate}
use std.graph.{Graph, Lit, Target, Tree, TwoSat, Visit, bfs, condense, dfs, dijkstra, peel, reachable, scc, topo}
use std.grid.{Dir4, Dir8, adj4, adj8, ray, step}
use std.heap.{Heap, MaxHeap, MinHeap}
use std.interval.{IntervalMap}
use std.io.{Wire, yes}
use std.lazy.{Flip}
use std.marks.{Marks}
use std.matrix.{Mat}
use std.math.{Ratio, Sieve, bit, ceil_div, digits, divmod, fixed, floor_quotients, gcd, inf, lcm, sign}
use std.mod.{Comb, Mod}
use std.prefix.{Diff, Prefix, scan}
use std.search.{bsearch_real, ternary_max, ternary_max_real, ternary_min, ternary_min_real}
use std.segtree.{SegTree}
use std.seq.{
    buckets,
    chain,
    chunks,
    chunks_padded,
    compress,
    cyclic_pairs,
    dedup,
    deduped,
    distinct,
    drop_while,
    equal_range,
    first_gt,
    flat_map,
    flatten,
    get_or_insert,
    group,
    insert,
    inverse,
    inversions,
    is_sorted,
    is_subsequence,
    iterate,
    last_lt,
    lis,
    next_permutation,
    select,
    pairs,
    partition,
    partition_point,
    prev_permutation,
    prod,
    reduce,
    remove,
    resized,
    reverse,
    rotate_left,
    rotate_right,
    rotated_left,
    rotated_right,
    runs,
    take_while,
    tally,
    windows,
    zip,
}
use std.sparse.{SparseTable}
use std.strings.{
    RollingHash,
    SuffixArray,
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
use std.tree.{JumpTree, LcaTable, Lift, Rooted}

