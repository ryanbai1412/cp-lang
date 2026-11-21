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

use std.algebra.{Action, Desc, Monoid, Ordering, cmp, desc, fold}
use std.bag.{BucketCount, SortedBag, SortedSet}
use std.bitset.{Bitset}
use std.deque.{Deque, LinkedList}
use std.dp.{LiChao, Line}
use std.dsu.{Dsu, DsuOf, PotentialDsu}
use std.fenwick.{Fenwick, Fenwick2D}
use std.flow.{Arc, FlowGraph}
use std.geometry.{ByAngle, Pt}
use std.graph.{Graph, Target, Tree, bfs, condense, dijkstra, peel, scc, topo}
use std.grid.{Dir4, Dir8, step}
use std.heap.{MaxHeap, MinHeap}
use std.interval.{IntervalMap}
use std.lazy.{AddMax, AddMin, AddSum, AssignMax, AssignMin, AssignSum, DualSegTree, LazySegTree}
use std.marks.{Marks}
use std.math.{Ratio, Sieve, bit, ceil_div, digits, divmod, fixed, gcd, lcm, sign}
use std.mod.{Comb, Mod}
use std.prefix.{Diff, Prefix}
use std.segtree.{SegTree}
use std.seq.{
    argsort,
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
    excl_fold,
    first_gt,
    flat_map,
    flatten,
    get_or_insert,
    group_by,
    insert,
    inverse,
    inversions,
    is_sorted,
    is_subsequence,
    iterate,
    last_lt,
    lis,
    next_greater,
    next_less,
    next_permutation,
    nth_element,
    pairs,
    partition,
    partition_point,
    prev_greater,
    prev_less,
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
    run_ranges,
    runs,
    scan,
    sliding_max,
    sliding_min,
    suffix_scan,
    take_while,
    tally,
    windows,
    with_prev,
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

