// CP Lang 0.2.2 language tables for fallback completion and grammar checks.
// `test/unit/languageData.test.ts` checks them against the compiler's prelude
// (`crates/cpl-check/src/prelude.rs`), `stdlib/*.cp` and `stdlib/prelude.cp`,
// and keeps `syntaxes/cp.tmLanguage.json` in lockstep with them.

/** 30 contestant keywords (including 3 literals) in CP Lang 0.2.2. */
export const CONTESTANT_KEYWORDS: readonly string[] = [
    'let', 'var', 'const', 'fn', 'struct', 'enum', 'domain', 'type', 'use', 'pub',
    'if', 'elif', 'else', 'for', 'while', 'in', 'match', 'return', 'break', 'continue',
    'and', 'or', 'not', 'assert', 'mut', 'self', 'unchecked',
    'true', 'false', 'none',
];

/** 4 library-author keywords in CP Lang 0.2.2. */
export const LIBRARY_KEYWORDS: readonly string[] = [
    'trait', 'impl', 'lends', 'yield',
];

/** All 34 reserved keywords in CP Lang 0.2.2. */
export const KEYWORDS: readonly string[] = [...CONTESTANT_KEYWORDS, ...LIBRARY_KEYWORDS];

/**
 * Words that are keywords only in one position, so they stay usable as names:
 * `x as T`, `yield from`, `invariant ...`, `law assoc(a, b) = ...`, `-> yields (v: V)`, `= do:`.
 */
export const CONTEXTUAL_KEYWORDS: readonly string[] = [
    'as', 'from', 'invariant', 'law', 'yields', 'do',
];

/**
 * Types the compiler itself knows (`crates/cpl-check/src/prelude.rs`). `Opt` (spelled `?T`)
 * and `Counter` (what `counts` returns) are left out: neither is written by name.
 */
export const PRELUDE_TYPES: readonly string[] = [
    'Bool', 'Bounds', 'Char', 'F64', 'HashMap', 'I128', 'Int', 'Interactor', 'Iter', 'List', 'Map',
    'Range', 'Set', 'Str', 'Subset', 'Sym', 'U64',
];

/** Every public type and trait declared in `stdlib/*.cp`, whether `stdlib/prelude.cp` imports it or not. */
export const STDLIB_TYPES: readonly string[] = [
    'Action', 'AhoCorasick', 'Arc', 'BagCursor', 'BeatsSegTree', 'Bitset', 'ByAngle', 'Centroids', 'Comb', 'Compressed',
    'ConnectedGraph', 'CostFlow', 'Dag', 'Deque', 'Desc', 'Diff1', 'Diff2', 'Digraph', 'Dir4', 'Dir8',
    'DisjointSparseTable', 'Dsu', 'DsuOf', 'DualFenwick', 'DualSegTree', 'ErasableMaxHeap', 'ErasableMinHeap', 'Fenwick', 'Fenwick2D', 'FlatMap',
    'FlowGraph', 'Graph', 'HalfPlane', 'Hash', 'Hash2D', 'HashVal', 'Hashable', 'IndexedMaxHeap', 'IndexedMinHeap', 'Interner',
    'IntervalMap', 'LazySegTree', 'LiChao', 'LinkCut', 'Line', 'Lit', 'Location', 'Mark', 'MarkMap', 'Marks', 'Mat',
    'MaxHeap', 'MaxSub', 'MergeSortTree', 'MinHeap', 'Mod', 'MonoDeque', 'Monoid', 'Neighbours', 'OrderedSet', 'Palindromes',
    'PalindromicTree', 'ParityDsu', 'Paths', 'Perm', 'PersistentSegTree', 'PotentialDsu', 'Prefix1', 'Prefix2', 'PrefixN', 'Pt',
    'Pt3', 'RangeFenwick', 'RangeKth', 'Reversible', 'Rng', 'RollbackArray', 'RollingHash', 'Rooted', 'SegTree', 'Sieve',
    'SimpleGraph', 'SortedBag', 'SosOver', 'SparseSegTree', 'SparseTable', 'SuffixArray', 'SuffixAutomaton', 'Target', 'TimeStep', 'TimeTree',
    'Tree', 'Trie', 'TwoSat', 'Vec', 'Version', 'Walk', 'WindowFold', 'XorBasis', 'XorFenwick',
];

/**
 * Functions a program can call without a `use`: the compiler's own (`prelude.rs`) and the
 * ones `stdlib/prelude.cp` imports.
 */
export const PRELUDE_VALUES: readonly string[] = [
    'abs', 'all', 'any', 'ask', 'atan2', 'bit_ceil', 'bit_width', 'ceil', 'copy', 'count', 'counts',
    'ctz', 'debug', 'first', 'floor', 'flush', 'isqrt', 'laws_hold', 'len', 'max', 'min',
    'popcount', 'prefix', 'print', 'rand', 'read', 'read_output', 'round', 'shuffle', 'some',
    'sort', 'sorted', 'span', 'sqrt', 'sum', 'prod', 'swap', 'trunc', 'bfs', 'bit', 'bsearch_first',
    'bsearch_last', 'ceil_div', 'compress', 'condense', 'deduped', 'desc', 'digits', 'dijkstra',
    'ends_with', 'excl_fold', 'find', 'fixed', 'fold', 'gcd', 'get_or_insert', 'insert', 'lcm',
    'lower', 'partition', 'peel', 'prefix1', 'prefix2', 'prefix_n', 'remove',
    'repeat', 'resized', 'runs', 'scan', 'scc', 'starts_with', 'topo', 'upper', 'digit',
    'find_all', 'step', 'windows', 'zip',
];

/** Public `stdlib/*.cp` functions that need a `use` (`use std.strings.{z_function}`). */
export const STDLIB_VALUES: readonly string[] = [
    'affine_sum', 'angle_less', 'any_segments_meet', 'area2', 'area2_128', 'arg', 'argmins',
    'articulation_points', 'bellman_ford', 'berlekamp_massey', 'bfs01', 'biconnected_components', 'bit_len', 'borders',
    'bridges',
    'bsearch_real', 'buckets', 'choose', 'choose2', 'chunk_by', 'chunks', 'circle_intersect',
    'circle_line_intersect', 'circle_polygon_area', 'circulation', 'circumcenter', 'close', 'closest_pair',
    'closest_pair2', 'components', 'compose', 'concavity_break', 'convex_hull', 'convex_hull_3d', 'convex_hull_idx',
    'convexity_break', 'convolve', 'convolve_and', 'convolve_int', 'convolve_mod', 'convolve_or',
    'convolve_xor', 'cross', 'cross128', 'cross3', 'crt', 'crt_all', 'cyclic_pairs', 'decrease', 'dedup',
    'det_real', 'diameter2', 'differences', 'dist',
    'dist2', 'dist2_128', 'dist2_3', 'divisors', 'divisors_from_factors', 'dnc_break', 'dot', 'dot128', 'dot3', 'dot3_128', 'euler_walk', 'eq_fields',
    'ext_gcd', 'factor', 'factor_big', 'find_cycle', 'first_gt', 'fixed_cost_paths', 'floor_sum', 'floyd_warshall',
    'general_matching', 'get_or_insert_with', 'global_min_cut', 'gomory_hu', 'guess_at', 'half_plane_intersect', 'hash_fields',
    'hull_extreme', 'hull_tangents', 'idempotent', 'increase', 'inv_mod', 'inverse', 'inverse_real', 'inversions',
    'is_palindrome', 'is_prime', 'is_sorted', 'knuth_break',
    'last_lt', 'lcp_array', 'line_dist', 'line_intersect', 'locate', 'locate_convex',
    'lower_hull', 'lt_fields', 'lyndon_factorization', 'manacher', 'max_bounded_flow', 'max_closure',
    'max_matching', 'max_weight_matching', 'merge_sorted', 'min_bounded_flow', 'min_enclosing_circle', 'min_rotation',
    'min_vertex_cover', 'min_weight_matching', 'minkowski_sum', 'minkowski_upper', 'mo_order',
    'mo_steps', 'mod_log', 'mod_sqrt', 'monge_break', 'mul_mod', 'newton', 'next_greater',
    'next_less', 'next_permutation', 'nth_element', 'ntt', 'on_segment', 'overlap', 'pairs',
    'period', 'perm_cycles', 'perm_pow', 'phi', 'poly_at', 'poly_degree', 'polygon_area', 'polygon_cut', 'pow_mod', 'prefix_fold',
    'prefix_function', 'prefix_matches', 'prev_greater', 'prev_less', 'prev_permutation',
    'primitive_root', 'project', 'rank_real', 'ray', 'recurrence_at', 'reflect', 'reverse', 'rfind',
    'rotate_left', 'rotate_right', 'run_ranges', 'same_dir', 'seg_dist',
    'segments_meet', 'show_fields', 'sieve_multiplicative', 'sign_eps', 'sliding_max', 'sliding_min', 'solve_real',
    'sos', 'splitmix64', 'suffix_array', 'suffix_fold', 'suffix_scan', 'swap_at',
    'take', 'ternary_max', 'ternary_max_real', 'ternary_min', 'ternary_min_real', 'turn',
    'two_color', 'two_edge_components', 
    'unimodality_break', 'update', 'upper_hull', 'vcross', 'vdist', 'vdot', 'vturn', 'with_prev',
    'z_function',
    'aliens', 'dnc_layer', 'knuth', 'monge_path', 'monotone_minima', 'parallel_bsearch', 'subset_sums',
];

/** Public `stdlib/*.cp` constants (no INF sentinel; use ?Int / none). */
export const PRELUDE_CONSTANTS: readonly string[] = [
    'AddMax', 'AddMin', 'AddSum', 'AffineSum', 'AssignMax', 'AssignMin', 'AssignSum',
];

export interface DecoratorInfo {
    name: string;
    /** Text inserted after `@`, as a snippet. */
    insert: string;
    detail: string;
}

/** File-level directives (`@!...`) in CP Lang 0.2.2; apply from that line down. */
export const FILE_DIRECTIVES: readonly DecoratorInfo[] = [
    {
        name: 'multitest',
        insert: 'multitest',
        detail: 'file-level directive: precompute above, loop t test cases below with fresh scope and @memo resets',
    },
    {
        name: 'interactive',
        insert: 'interactive',
        detail: 'file-level directive: every print is written out at once, for an interactive judge',
    },
];

/** Item attributes (`@...`): 8 contestant + 3 library-author decorators in CP Lang 0.2.2. */
export const DECORATORS: readonly DecoratorInfo[] = [
    { name: 'memo', insert: 'memo', detail: 'memoize a function (auto-dispatches flat vs salted hash table)' },
    { name: 'multitest', insert: 'multitest', detail: 'legacy item-attribute spelling of `@!multitest`' },
    { name: 'brute', insert: 'brute(${1:main})', detail: 'reference brute-force implementation for `cpl stress`' },
    { name: 'tags', insert: "tags('${1:.}', '${2:#}')", detail: 'compact character-set tag for grid cells' },
    { name: 'gen', insert: 'gen', detail: 'custom whole-input generator for `cpl stress`' },
    { name: 'checker', insert: 'checker', detail: 'constructive or multi-answer validator for `cpl stress`' },
    { name: 'interactor', insert: 'interactor', detail: 'local interactive judge taking `sol: Interactor`' },
    { name: 'grader', insert: 'grader', detail: 'IOI grader-provided function or struct' },
    { name: 'get', insert: 'get', detail: 'library getter called without parentheses' },
    { name: 'test', insert: 'test', detail: 'unit test function run by `cpl test`' },
    { name: 'law', insert: 'law ${1:disjoint}(${2:i}, ${3:j}) = ${4:i != j}', detail: 'library disjointness or algebraic law annotation' },
];
