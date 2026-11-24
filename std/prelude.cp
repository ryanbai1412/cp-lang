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

use std.algebra.{
    Each,
    Action,
    Commutative,
    Desc,
    Group,
    Idempotent,
    Monoid,
    Op,
    Ordering,
    Selective,
    Semiring,
    add,
    affine,
    assign,
    cmp,
    desc,
    each,
    flip,
    fold,
    minmax,
    pow,
}
use std.bag.{SortedBag, SortedMap, SortedSet}
use std.beats.{BeatsSegTree}
use std.bitset.{Bitset}
use std.deque.{Deque}
use std.derive.{Hash}
use std.dp.{LiChao, Line, LineContainer, MonotoneHull, aliens}
use std.dsu.{Dsu, DsuOf, ParityDsu, PotentialDsu}
use std.fenwick.{Fenwick, Fenwick2D}
use std.flow.{Arc, FlowGraph, max_closure}
use std.geometry.{ByAngle, Location, Pt, cross, dot, locate, turn}
use std.ntt.{Curved, convolve}
use std.graph.{
    Functional,
    Graph,
    Lit,
    Target,
    Tree,
    TwoSat,
    Visit,
    bfs,
    components,
    condense,
    dfs,
    dijkstra,
    floyd_warshall,
    kruskal,
    peel,
    reachable,
    scc,
    topo,
}
use std.grid.{Dir4, Dir8, adj4, adj8, ray, step}
use std.heap.{Heap, MaxHeap, MinHeap, Settle}
use std.interval.{IntervalMap}
use std.io.{Wire, io, read_all, yes}
use std.lazy.{Flip}
use std.marks.{Marks}
use std.matrix.{Mat}
use std.math.{
    BigInt,
    Sign,
    Ratio,
    Sieve,
    bit,
    bit_len,
    ceil_div,
    choose2,
    digits,
    divisors,
    divmod,
    fixed,
    floor_quotients,
    gcd,
    inf,
    isqrt,
    lcm,
    phi,
    pow_mod,
    sign,
    to_str,
}
use std.mod.{Comb, Mod}
use std.opt.{and_then, is_none_or, is_some_and}
use std.prefix.{Diff, Prefix, scan}
use std.search.{
    bsearch_real,
    split_best,
    ternary_max,
    ternary_max_real,
    ternary_min,
    ternary_min_real,
}
use std.segtree.{SegTree}
use std.window.{Sweep, Track, Undo, Undoing, Window, track, undoing}
use std.seq.{
    Compressed,
    Rev,
    RingSlice,
    Slice,
    Stepped,
    Perm,
    buckets,
    chain,
    chunks,
    compress,
    cyclic_pairs,
    dedup,
    deduped,
    distinct,
    drop_while,
    equal_range,
    find,
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
    adjacent,
    Anchor,
    Ties,
    pairs,
    partition,
    prev_permutation,
    product,
    all,
    any,
    count,
    prod,
    step,
    sum,
    reduce,
    remove,
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
use std.hashmap.{Counter}
use std.pair.{Pair}
use std.poly.{eval}
use std.sparse.{SparseTable}
use std.subset.{Subset}
use std.strings.{
    Palindromes,
    RollingHash,
    SuffixArray,
    borders,
    common_prefix_len,
    common_suffix_len,
    digit,
    ends_with,
    find_all,
    lower,
    lyndon_factorization,
    manacher,
    min_rotation,
    overlap,
    period,
    prefix_function,
    repeat,
    starts_with,
    upper,
    z_function,
}
use std.tree.{JumpTree, KruskalTree, LcaTable, Lift, PreferredPaths, Rooted, kruskal_tree}
