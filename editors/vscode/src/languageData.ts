// CP Lang 0.2.2 language tables for fallback completion and grammar checks.
// `test/unit/languageData.test.ts` checks them against the compiler's prelude
// (`crates/cpl-check/src/prelude.rs`), `spec/std/*.cp` and `spec/std/prelude.cp`,
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
    'Bool', 'Char', 'Cut', 'F64', 'HashMap', 'HashSet', 'I128', 'Indexed', 'Int', 'Interactor', 'Iter',
    'List', 'Map', 'Range', 'Seq', 'Set', 'SortedRange', 'Str', 'Subset', 'Sym', 'U64',
];

/** Every public type and trait declared in `spec/std/*.cp`, whether `spec/std/prelude.cp` imports it or not. */
export const STDLIB_TYPES: readonly string[] = [
    'Action', 'Affine', 'AhoCorasick', 'Anchor', 'Arc', 'Arena', 'ArithDiff', 'AugSortedBag', 'AugSortedSet',
    'BagCursor', 'Barrett', 'BeatsSegTree', 'BigInt', 'BitMat', 'Bitset', 'BucketCount', 'ByAngle',
    'ByAngleFrom', 'Centroids', 'Circle', 'ColorMatroid', 'Comb', 'Commutative', 'Complement', 'Compressed',
    'Concave', 'ConnectedGraph', 'Convex', 'CostFlow', 'Counter', 'Curved', 'Dag', 'Deque', 'Desc', 'Diff',
    'Digraph', 'Dir4', 'Dir8', 'DirichletSum', 'Disjoint', 'DisjointSparseTable', 'Dsu', 'DsuOf',
    'DualFenwick', 'DynBitset', 'DynamicAhoCorasick', 'DynamicHash', 'DynamicOrderTree', 'Edge', 'EdgeArc',
    'Edged', 'EdgedNext', 'ErasableHeap', 'EulerTourTree', 'Events', 'Fenwick', 'Fenwick2D', 'FillOrder',
    'FillPrefix', 'Filtered', 'FlatMap', 'Flip', 'FlowGraph', 'Functional', 'Graph', 'GraphicMatroid',
    'Group', 'HalfPlane', 'HasId', 'HasInv', 'Hash2D', 'Hash61', 'HashVal', 'Hashable', 'Heap',
    'HistoricBeatsSegTree', 'Id', 'Idempotent', 'IncrementalBridges', 'IndexedHeap', 'Interner',
    'IntervalMap', 'IntervalSet', 'Items', 'JumpTree', 'Keys', 'KmpAutomaton', 'KruskalTree', 'Layers',
    'LazyEulerTourTree', 'LazyLinkCut', 'LazyTreap', 'LcaTable', 'LiChao', 'Lift', 'Line', 'LineContainer',
    'LinearRmq', 'LinkCut', 'LinkedList', 'Lit', 'Location', 'LongPaths', 'Mapped', 'Mark', 'MarkMap',
    'Marks', 'Mat', 'MatPow', 'Matching', 'Matroid', 'MaxCount', 'MaxHeap', 'MaxSub', 'MeldHeap', 'MergeSeg',
    'MergeSortTree', 'MinCount', 'MinHeap', 'MinMaxHeap', 'Mod', 'ModComb', 'MongePath', 'MonoStack',
    'Monoid', 'MonotoneHull', 'Next', 'OnlineHull', 'OnlineLis', 'Only', 'Op', 'Ordering', 'PMultiset',
    'Pair', 'Palindromes', 'PalindromicTree', 'ParityDsu', 'Paths', 'Perm', 'PermKind', 'PermNode',
    'PermTree', 'PersistentArray', 'PersistentDsu', 'PersistentLazySegTree', 'PersistentSegTree', 'Plane3',
    'Poly', 'PotentialDsu', 'PreferredPaths', 'Prefix', 'PrefixXorBasis', 'PrimePowerComb', 'Pt', 'Pt3',
    'RangeKth', 'Ratio', 'Rebased', 'Rect', 'RelaxedConvolve', 'Rev', 'Reversible', 'Reweighted', 'Ring',
    'RingRange', 'RingSlice', 'Rng', 'RollbackArray', 'RollingHash', 'RollingHash61', 'Rooted', 'Rope',
    'SegTree', 'SegTree2D', 'Selective', 'SemiRelaxedConvolve', 'Semigroup', 'Semiring', 'Settle', 'Sieve',
    'Sign', 'SimpleGraph', 'Slice', 'SortedBag', 'SortedMap', 'SortedRange', 'SortedSet', 'SosOver',
    'SparseFenwick2D', 'SparseIntervalMap', 'SparseIntervalSet', 'SparseSet', 'SparseTable', 'SparseTable2D',
    'Step', 'SubproductTree', 'SubseqStep', 'Subset', 'SuffixArray', 'SuffixAutomaton', 'SuffixTreeEdge',
    'Sweep', 'SweepAdd', 'Target', 'TimeStep', 'TimeTree', 'Track', 'Tracked', 'Tree', 'Trie', 'TwoSat',
    'Undo', 'Undoing', 'Vec', 'Vec3', 'Version', 'VirtualTree', 'Visit', 'Walk', 'WaveletMatrix', 'Weighted',
    'WeightedNext', 'Window', 'Wire', 'XorBasis', 'XorTrie',
];

/**
 * Functions a program can call without a `use`: the compiler's own (`prelude.rs`) and the
 * ones `spec/std/prelude.cp` imports.
 */
export const PRELUDE_VALUES: readonly string[] = [
    'abs', 'adj4', 'adj8', 'adjacency', 'adjacent', 'aliens', 'all', 'any', 'ask', 'atan2', 'bfs', 'bit',
    'bit_ceil', 'bit_width', 'bsearch_real', 'buckets', 'ceil', 'ceil_div', 'chain', 'choose2', 'chunks',
    'cmp', 'common_prefix_len', 'common_suffix_len', 'compress', 'condense', 'convolve', 'copy', 'count',
    'counts', 'ctz', 'cyclic_pairs', 'debug', 'dedup', 'deduped', 'desc', 'dfs', 'digit', 'digits',
    'dijkstra', 'distinct', 'divmod', 'drop_while', 'each', 'ends_with', 'equal_range', 'eval', 'find',
    'find_all', 'first', 'first_gt', 'fixed', 'flat_map', 'flatten', 'floor', 'floor_quotients',
    'floyd_warshall', 'flush', 'fold', 'gcd', 'get_or_insert', 'group', 'insert', 'inverse', 'inversions',
    'is_none_or', 'is_some_and', 'is_sorted', 'is_subsequence', 'isqrt', 'iterate', 'kruskal', 'kruskal_tree',
    'last_lt', 'laws_hold', 'lcm', 'len', 'lis', 'locate', 'lower', 'max', 'max_closure', 'min', 'minmax',
    'next_permutation', 'pairs', 'partition', 'partition_point', 'peel', 'phi', 'popcount', 'pow_mod',
    'prefix', 'prev_permutation', 'print', 'prod', 'rand', 'ray', 'reach_sets', 'reachable', 'read', 'read_all',
    'read_output', 'reduce', 'remove', 'repeat', 'reverse', 'rotate_left', 'rotate_right', 'rotated_left',
    'rotated_right', 'round', 'runs', 'scan', 'scc', 'select', 'shuffle', 'sign', 'some', 'sort', 'sorted',
    'span', 'split_best', 'sqrt', 'starts_with', 'step', 'sum', 'swap', 'take_while', 'tally', 'ternary_max',
    'ternary_max_real', 'ternary_min', 'ternary_min_real', 'to_str', 'topo', 'track', 'trunc', 'undoing',
    'upper', 'windows', 'yes', 'zip',
];

/** Public `spec/std/*.cp` functions that need a `use` (`use std.strings.{z_function}`). */
export const STDLIB_VALUES: readonly string[] = [
    'adjacency', 'adjacent_diff', 'all_some', 'angle_between', 'angle_less', 'any_segments_meet',
    'any_segments_meet_idx', 'arborescence', 'area2', 'area2_128', 'arg', 'argmax', 'argmin', 'backtrack',
    'bell_table', 'bellman_ford', 'bernoulli_table', 'best_matching', 'best_rational', 'bfs01', 'bit_len',
    'bit_subsets', 'block_cut_tree', 'blocks', 'borders', 'boruvka', 'bostan_mori', 'bridges', 'bsearch',
    'carmichael', 'cartesian_tree', 'cdq', 'ceil_sqrt', 'chirp_z', 'choose', 'choose2', 'chromatic_number',
    'chunks_padded', 'circle_circle_area', 'circle_circle_tangents', 'circle_intersect',
    'circle_line_intersect', 'circle_polygon_area', 'circle_tangents', 'circles_union_area', 'circulation',
    'circumcenter', 'clip', 'clone', 'close', 'closest_pair', 'closest_pair2', 'closure', 'collect',
    'combinations', 'complement', 'components', 'compose', 'compose_inv', 'continued_fraction', 'convex_hull',
    'convex_hull_3d', 'convex_hull_3d_idx', 'convex_hull_idx', 'convex_polygon_intersect',
    'convex_polygons_overlap', 'convolve_all', 'count_paths', 'counting_sort', 'covering', 'crt', 'crt_all',
    'cut_vertices', 'cycle', 'dag_fold', 'de_bruijn', 'delaunay', 'deriv', 'det_real', 'dial', 'diameter2',
    'dinkelbach', 'dirichlet_convolve', 'dirichlet_inv', 'div_mod', 'divisors', 'divisors_from_factors',
    'dnc_layer', 'dominators', 'drop', 'drop_last', 'each_divisor', 'edge_coloring', 'euler_walk', 'eval_all',
    'exp', 'ext_gcd', 'factor', 'farthest_pair', 'filter', 'filter_map', 'find_all_wildcard', 'find_cycle',
    'first_some', 'first_true', 'fixed_cost_paths', 'flip_action', 'floor_sum', 'from_prufer', 'fwht',
    'gallop_first', 'gcd_poly', 'general_matching', 'get_or_insert_with', 'girth', 'global_min_cut',
    'gomory_hu', 'half_plane_intersect', 'halve', 'hull_extreme', 'hull_line_intersect', 'hull_tangents',
    'hull_width', 'idempotent', 'incenter', 'index_pairs', 'integral', 'interpolate', 'inv', 'inv_mod',
    'inverse_real', 'invert', 'iroot', 'is_palindrome', 'is_permutation', 'is_prime', 'isqrt', 'jacobi',
    'johnson', 'join', 'k_shortest', 'keyed', 'knuth', 'kway_merge', 'lagrange_at', 'last_true', 'lcs',
    'lcs_all', 'line_dist', 'line_intersect', 'line_plane_intersect', 'linear_congruence',
    'linear_diophantine', 'lis_ends', 'locate_convex', 'log', 'lower_hull', 'lower_hull_idx',
    'lyndon_factorization', 'lyndon_runs', 'majority', 'manacher', 'manhattan_mst', 'map', 'masks_of_size',
    'matroid_intersection', 'max', 'max_bounded_flow', 'max_clique', 'max_count', 'max_k', 'max_matching',
    'max_rotation', 'max_weight_general_matching', 'max_weight_matching', 'maximal', 'mean', 'merge_sorted',
    'min', 'min_bounded_flow', 'min_bounding_box', 'min_cost_circulation', 'min_count',
    'min_enclosing_circle', 'min_k', 'min_mean_cycle', 'min_mod_in_range', 'min_rotation', 'min_vertex_cover',
    'min_weight_matching', 'minimal', 'minkowski_lower', 'minkowski_sum', 'minkowski_upper', 'mo', 'mo_order',
    'mo_order_hilbert', 'mo_rollback', 'mo_steps', 'mo_with_updates', 'mod_kth_root', 'mod_log', 'mod_sqrt',
    'monge_path', 'monge_paths', 'monotone_minima', 'mul_mod', 'mult_order', 'negative_cycle', 'next_same',
    'next_smaller', 'nth', 'ntt', 'odd_cycle', 'offline_over', 'on_segment', 'only', 'outermost', 'overlap',
    'parallel_bsearch', 'parse', 'parse_int', 'partial_inverse', 'partition_in_place', 'partition_table',
    'period', 'perm_cycles', 'perm_order', 'perm_pow', 'perm_rank', 'perm_sign', 'perm_unrank',
    'permutations', 'pick_boundary', 'pick_interior', 'pieces', 'planar_faces', 'point_action',
    'point_location_planar', 'polygon_area', 'polygon_centroid', 'polygon_cut', 'polygon_kernel',
    'polygon_triangulate', 'polyhedron_volume6', 'pop_if', 'pop_while', 'pow', 'pow_mod_poly', 'power_sums',
    'prefix_excl', 'prefix_fold', 'prefix_function', 'prefix_matches', 'present', 'prev_same', 'prev_smaller',
    'prim', 'prime_count', 'primes_in', 'primitive_root', 'prod_linear', 'product', 'project', 'prufer',
    'radix_sort', 'rank_real', 'ray_hits_segment', 'record_highs', 'rects_union_area', 'reflect', 'rekey',
    'replace', 'resized', 'rfind', 'rsearch', 'runs_linked', 'same_dir', 'sample_points_shift', 'scc_times',
    'search', 'seg_dist', 'segment_intersect', 'segments_meet', 'shift_and', 'sieve_multiplicative',
    'sign_eps', 'simplex', 'smallest_topo', 'smawk', 'solve_real', 'sort', 'sorted', 'sorted_diff',
    'sorted_intersect', 'sorted_union', 'sos', 'span', 'spanning_forest', 'spans', 'sparse_det',
    'sparse_solve', 'sphere_dist', 'split', 'split_once', 'split_whitespace', 'splitmix64', 'sqrt',
    'stable_matching', 'stable_partition_in_place', 'stirling1_col', 'stirling1_row', 'stirling2_col',
    'stirling2_row', 'submasks', 'subsequences', 'subset_exp', 'subset_folds', 'subset_log', 'subset_sums',
    'suffix_array', 'suffix_fold', 'suffix_tree_edges', 'sum_mobius', 'sum_multiplicative', 'sum_phi',
    'supermasks', 'swap_at', 'symmetric', 'take', 'take_all', 'take_last', 'taylor_shift', 'to_lower',
    'to_upper', 'transposed', 'transposed_mul', 'triangles', 'trim', 'trim_end', 'trim_start', 'turn',
    'turn_eps', 'two_color', 'two_edge_components', 'update', 'upper_hull', 'upper_hull_idx', 'valleys',
    'voronoi_cells', 'with_domain', 'with_next', 'z_function', 'zip_exact',
];

/** Public `spec/std/*.cp` constants the prelude imports (no INF sentinel; use ?Int / none). */
export const PRELUDE_CONSTANTS: readonly string[] = [
    'add', 'affine', 'assign', 'flip', 'inf', 'io',
];

/** Public `spec/std/*.cp` constants that need a `use`. */
export const STDLIB_CONSTANTS: readonly string[] = [
    'PI', 'subset',
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
