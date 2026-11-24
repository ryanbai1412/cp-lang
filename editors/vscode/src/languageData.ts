// CP Lang 0.2.2 language tables for fallback completion and grammar checks.
// `test/unit/languageData.test.ts` checks them against the compiler's prelude
// (`crates/cpl-check/src/prelude.rs`), `std/*.cp` and `std/prelude.cp`,
// and keeps `syntaxes/cp.tmLanguage.json` in lockstep with them.

/** 29 contestant keywords (including 2 literals) in CP Lang 0.2.2. */
export const CONTESTANT_KEYWORDS: readonly string[] = [
    'let', 'var', 'const', 'fn', 'struct', 'enum', 'domain', 'type', 'use', 'pub',
    'if', 'elif', 'else', 'for', 'while', 'in', 'match', 'return', 'break', 'continue',
    'and', 'or', 'not', 'assert', 'mut', 'self', 'unchecked',
    'true', 'false',
];

/** 4 library-author keywords in CP Lang 0.2.2. */
export const LIBRARY_KEYWORDS: readonly string[] = [
    'trait', 'impl', 'lends', 'yield',
];

/** All 33 reserved keywords in CP Lang 0.2.2. */
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
    'Bool', 'Char', 'Cut', 'F64', 'HashMap', 'HashSet', 'I128', 'Indexed', 'Int', 'Interactor',
    'Iter', 'List', 'Map', 'Range', 'Seq', 'Set', 'SortedRange', 'Str', 'Subset', 'Sym', 'U64',
];

/** Every public type and trait declared in `std/*.cp`, whether `std/prelude.cp` imports it or not. */
export const STDLIB_TYPES: readonly string[] = [
    'Action', 'AddAction', 'Affine', 'AffineAction', 'AhoCorasick', 'Anchor', 'Arc', 'Arena',
    'ArithDiff', 'AssignAction', 'AugSortedBag', 'AugSortedSet', 'BagCursor', 'Barrett',
    'BeatsSegTree', 'Bidir', 'BigInt', 'BitMat', 'BitTrie', 'Bitset', 'BucketCount', 'ByAngle',
    'ByAngleFrom', 'Centroids', 'Circle', 'ColorMatroid', 'Comb', 'Commutative', 'Complement',
    'Compressed', 'Concave', 'ConnectedGraph', 'Convex', 'CostFlow', 'Counter', 'Curved', 'Dag',
    'Deque', 'Desc', 'Diff', 'Digraph', 'Dir4', 'Dir8', 'DirichletSum', 'Disjoint',
    'DisjointSparseTable', 'Dsu', 'DsuNode', 'DsuOf', 'DualFenwick', 'DynBitset',
    'DynamicAhoCorasick', 'DynamicHash', 'DynamicOrderTree', 'Each', 'Edge', 'EdgeArc', 'Edged',
    'EdgedNext', 'ErasableHeap', 'EulerTourTree', 'Events', 'FastProduct', 'Fenwick', 'Fenwick2D',
    'FillOrder', 'FillPrefix', 'Filtered', 'FlatMap', 'Flip', 'FlipAction', 'FlowGraph',
    'Functional', 'Graph', 'GraphicMatroid', 'GridCols', 'GridRows', 'Group', 'HalfPlane', 'HasId',
    'HasInv', 'Hash', 'Hash2D', 'Hash61', 'HashVal', 'Hashable', 'Heap', 'HistoricBeatsSegTree',
    'Id', 'Idempotent', 'IncrementalBridges', 'IndexOp', 'Indexed', 'IndexedHeap', 'Interner',
    'IntervalMap', 'IntervalSet', 'Items', 'JumpTree', 'Keyed', 'Keys', 'KmpAutomaton',
    'KruskalTree', 'Layers', 'LazyEulerTourTree', 'LazyLinkCut', 'LazyTreap', 'LcaTable', 'LiChao',
    'Lift', 'Line', 'LineContainer', 'LinearRmq', 'LinkCut', 'LinkedList', 'Lit', 'Location',
    'LongPaths', 'Mapped', 'Mark', 'MarkMap', 'Marks', 'Mat', 'MatPow', 'Matching', 'Matroid',
    'MaxCount', 'MaxHeap', 'MaxSub', 'MaxSubAt', 'Meld', 'MeldHeap', 'MeldHeaps', 'MergeSeg',
    'MergeSegs', 'MergeSortTree', 'MinCount', 'MinHeap', 'MinMaxHeap', 'Mod', 'ModComb',
    'MongePath', 'MonoStack', 'Monoid', 'MonotoneHull', 'Next', 'OnlineHull', 'OnlineLis', 'Only',
    'Op', 'Ordering', 'PArena', 'PMultiset', 'Pair', 'Palindromes', 'PalindromicTree', 'ParityDsu',
    'Paths', 'Perm', 'PermKind', 'PermNode', 'PermTree', 'PersistentArray', 'PersistentDsu',
    'PersistentLazySegTree', 'PersistentSegTree', 'Plane3', 'PointSet', 'Poly', 'PotentialDsu',
    'PreferredPaths', 'Prefix', 'PrefixXorBasis', 'PrimePowerComb', 'Pt', 'Pt3', 'RangeFenwick',
    'RangeKth', 'Ratio', 'Rebased', 'Rect', 'RelaxedConvolve', 'Rev', 'Reversible', 'Reweighted',
    'Ring', 'RingRange', 'RingSlice', 'Rng', 'RollbackArray', 'RollingHash', 'RollingHash61',
    'Rooted', 'Rope', 'SegTree', 'SegTree2D', 'Selective', 'SemiRelaxedConvolve', 'Semigroup',
    'Semiring', 'Sep', 'Seq', 'Settle', 'Sieve', 'Sign', 'SimpleGraph', 'Sized', 'Slice',
    'SortedBag', 'SortedMap', 'SortedRange', 'SortedSet', 'SosOver', 'SparseFenwick2D',
    'SparseIntervalMap', 'SparseIntervalSet', 'SparseSet', 'SparseTable', 'SparseTable2D', 'Step',
    'StepKeys', 'Stepped', 'SubproductTree', 'SubseqStep', 'Subset', 'SuffixArray',
    'SuffixAutomaton', 'SuffixTreeEdge', 'Sweep', 'SweepAdd', 'Target', 'Ties', 'TimeStep',
    'TimeTree', 'Track', 'Tracked', 'Tree', 'Trie', 'TwoSat', 'Undo', 'Undoing', 'Vec', 'Vec3',
    'Version', 'VirtualTree', 'Visit', 'Walk', 'WaveletMatrix', 'Weighted', 'WeightedNext',
    'Window', 'Wire', 'XorBasis', 'XorFenwick', 'XorTrie',
];

/**
 * Functions a program can call without a `use`: the compiler's own (`prelude.rs`) and the
 * ones `std/prelude.cp` imports.
 */
export const PRELUDE_VALUES: readonly string[] = [
    'abs', 'acos', 'adj4', 'adj8', 'adjacency', 'adjacent', 'aliens', 'all', 'and_then', 'any',
    'asin', 'ask', 'atan', 'atan2', 'bfs', 'bit', 'bit_ceil', 'bit_len', 'bit_width', 'borders',
    'bsearch_real', 'buckets', 'cbrt', 'ceil', 'ceil_div', 'chain', 'choose2', 'chunks', 'cmp',
    'common_prefix_len', 'common_suffix_len', 'components', 'compress', 'condense', 'convolve',
    'copy', 'cos', 'count', 'counts', 'cross', 'ctz', 'cyclic_pairs', 'debug', 'dedup', 'deduped',
    'desc', 'dfs', 'digit', 'digits', 'dijkstra', 'distinct', 'divisors', 'divmod', 'dot',
    'drop_while', 'each', 'ends_with', 'equal_range', 'eval', 'exp', 'find', 'find_all', 'first',
    'first_gt', 'fixed', 'flat_map', 'flatten', 'floor', 'floor_quotients', 'floyd_warshall',
    'flush', 'fold', 'gcd', 'gen', 'get_or_insert', 'group', 'hypot', 'insert', 'inverse',
    'inversions', 'is_none_or', 'is_some_and', 'is_sorted', 'is_subsequence', 'isqrt', 'iterate',
    'kruskal', 'kruskal_tree', 'last_lt', 'laws_hold', 'lcm', 'len', 'lis', 'ln', 'locate', 'log10',
    'log2', 'lower', 'lyndon_factorization', 'manacher', 'max', 'max_closure', 'min',
    'min_rotation', 'minmax', 'next_permutation', 'overlap', 'pairs', 'partition',
    'partition_point', 'peel', 'period', 'phi', 'popcount', 'pow', 'pow_mod', 'prefix',
    'prefix_function', 'prev_permutation', 'print', 'prod', 'product', 'rand', 'ray', 'reachable',
    'read', 'read_all', 'read_line', 'read_output', 'reduce', 'remove', 'repeat', 'reverse',
    'rotate_left', 'rotate_right', 'rotated_left', 'rotated_right', 'round', 'runs', 'scan', 'scc',
    'select', 'shuffle', 'sign', 'sin', 'some', 'sort', 'sorted', 'span', 'split_best', 'sqrt',
    'starts_with', 'step', 'sum', 'swap', 'take_while', 'tally', 'tan', 'ternary_max',
    'ternary_max_real', 'ternary_min', 'ternary_min_real', 'to_str', 'topo', 'track', 'trunc',
    'turn', 'undoing', 'upper', 'windows', 'yes', 'z_function', 'zip',
];

/** Public `std/*.cp` functions that need a `use` (`use std.strings.{z_function}`). */
export const STDLIB_VALUES: readonly string[] = [
    'adjacency', 'adjacent_diff', 'all_some', 'angle_between', 'angle_less', 'any_segments_meet',
    'any_segments_meet_idx', 'arborescence', 'arc_len', 'area2', 'area2_128', 'arg', 'argmax',
    'argmin', 'argmins', 'backtrack', 'bell_table', 'bellman_ford', 'berlekamp_massey',
    'bernoulli_table', 'best_matching', 'best_rational', 'bfs01', 'bit_subsets', 'block_cut_tree',
    'blocks', 'boruvka', 'bostan_mori', 'bridges', 'bsearch', 'carmichael', 'cartesian_tree', 'cdq',
    'ceil_sqrt', 'chirp_z', 'choose', 'chromatic_number', 'chunks_padded', 'circle_circle_area',
    'circle_circle_tangents', 'circle_intersect', 'circle_line_intersect', 'circle_polygon_area',
    'circle_tangents', 'circles_union_area', 'circulation', 'circumcenter', 'clip', 'clone',
    'close', 'closest_pair', 'closest_pair2', 'closure', 'collect', 'combinations', 'complement',
    'compose', 'compose_inv', 'concavity_break', 'continued_fraction', 'convex_hull',
    'convex_hull_3d', 'convex_hull_3d_idx', 'convex_hull_idx', 'convex_polygon_intersect',
    'convex_polygons_overlap', 'convexity_break', 'convolve_all', 'count_in', 'count_paths',
    'counting_sort', 'covering', 'crt', 'crt_all', 'cut_vertices', 'cycle', 'dag_fold', 'de_bruijn',
    'decrease', 'deg', 'delaunay', 'deriv', 'det_real', 'dfs_tree', 'dial', 'diameter2',
    'differences', 'dinkelbach', 'dirichlet_convolve', 'dirichlet_inv', 'div_mod',
    'divisors_from_factors', 'dnc_break', 'dnc_layer', 'dominators', 'drop', 'drop_last',
    'each_divisor', 'edge_coloring', 'eq_fields', 'euler_walk', 'eval_all', 'exp', 'ext_gcd',
    'factor', 'farthest_pair', 'filter', 'filter_map', 'find_all_wildcard', 'find_cycle',
    'first_some', 'first_true', 'fixed_cost_paths', 'flip_action', 'floor_sum', 'from_prufer',
    'fwht', 'gallop_first', 'gcd_poly', 'general_matching', 'girth', 'global_min_cut', 'gomory_hu',
    'guess_at', 'half_plane_intersect', 'halve', 'has', 'hash_fields', 'hull_extreme',
    'hull_line_intersect', 'hull_tangents', 'hull_width', 'idempotent', 'incenter', 'increase',
    'index_pairs', 'integral', 'interpolate', 'intersect', 'inv', 'inv_mod', 'inverse_real',
    'invert', 'iroot', 'is_palindrome', 'is_permutation', 'is_prime', 'jacobi', 'johnson', 'join',
    'k_shortest', 'keyed', 'knuth', 'knuth_break', 'kway_merge', 'lagrange_at', 'last_true', 'lcs',
    'lcs_all', 'line_dist', 'line_intersect', 'line_plane_intersect', 'linear_congruence',
    'linear_diophantine', 'lis_ends', 'locate_convex', 'log', 'lower_hull', 'lower_hull_idx',
    'lt_fields', 'lyndon_runs', 'majority', 'manhattan_mst', 'map', 'masks_of_size',
    'matroid_intersection', 'max', 'max_bounded_flow', 'max_clique', 'max_count', 'max_k',
    'max_matching', 'max_rotation', 'max_weight_general_matching', 'max_weight_matching', 'maximal',
    'mean', 'merge_sorted', 'min', 'min_bounded_flow', 'min_bounding_box', 'min_cost_circulation',
    'min_count', 'min_enclosing_circle', 'min_k', 'min_mean_cycle', 'min_mod_in_range',
    'min_vertex_cover', 'min_weight_matching', 'minimal', 'minkowski_lower', 'minkowski_sum',
    'minkowski_upper', 'mo', 'mo_order', 'mo_order_hilbert', 'mo_rollback', 'mo_steps',
    'mo_with_updates', 'mod_kth_root', 'mod_log', 'mod_sqrt', 'monge_break', 'monge_path',
    'monge_paths', 'monotone_minima', 'mul_mod', 'mult_order', 'negative_cycle', 'newton', 'next',
    'next_same', 'next_smaller', 'nth', 'ntt', 'odd_cycle', 'offline_over', 'on_segment', 'only',
    'opt', 'outermost', 'parallel_bsearch', 'parse', 'parse_int', 'partial_inverse',
    'partition_in_place', 'partition_table', 'peak', 'perm_cycles', 'perm_order', 'perm_pow',
    'perm_rank', 'perm_sign', 'perm_unrank', 'permutations', 'pick_boundary', 'pick_interior',
    'pieces', 'planar_faces', 'point_action', 'point_location_planar', 'poly_at', 'poly_degree',
    'polygon_area', 'polygon_centroid', 'polygon_cut', 'polygon_kernel', 'polygon_triangulate',
    'polyhedron_volume6', 'pop', 'pop_if', 'pop_while', 'pow_mod_poly', 'power_sums', 'prefix_fold',
    'prefix_matches', 'present', 'prev_same', 'prev_smaller', 'prim', 'prime_count', 'primes_in',
    'primitive_root', 'prod_linear', 'project', 'prufer', 'push', 'radix_sort', 'rank_real',
    'ray_hits_segment', 'reach_sets', 'read_fields', 'record_highs', 'rects_union_area',
    'recurrence_at', 'reflect', 'rekey', 'replace', 'resized', 'rfind', 'rsearch', 'same_dir',
    'sample_points_shift', 'scc_times', 'search', 'seg_dist', 'segment_intersect', 'segments_meet',
    'shift_and', 'show_fields', 'sieve_multiplicative', 'sign_eps', 'simplex', 'skip_while',
    'smallest_topo', 'smawk', 'solve_real', 'sort', 'sorted', 'sorted_diff', 'sorted_intersect',
    'sorted_union', 'sos', 'span', 'spanning_forest', 'spanning_tree', 'spans', 'sparse_det',
    'sparse_solve', 'sphere_dist', 'split', 'split_once', 'split_whitespace', 'splitmix64', 'sqrt',
    'stable_matching', 'stable_partition_in_place', 'stirling1_col', 'stirling1_row',
    'stirling2_col', 'stirling2_row', 'submasks', 'subsequences', 'subset_exp', 'subset_folds',
    'subset_log', 'subset_sums', 'subtract', 'suffix_array', 'suffix_fold', 'suffix_tree_edges',
    'sum_mobius', 'sum_multiplicative', 'sum_phi', 'supermasks', 'swap_at', 'symmetric', 'take',
    'take_all', 'take_last', 'taylor_shift', 'to_lower', 'to_upper', 'transpose', 'transposed',
    'transposed_mul', 'triangles', 'trim', 'trim_end', 'trim_start', 'turn_eps', 'two_color',
    'two_edge_components', 'unimodality_break', 'union', 'unzip', 'update', 'upper_hull',
    'upper_hull_idx', 'valley', 'valleys', 'voronoi_cells', 'with_domain', 'with_next', 'zip_exact',
];

/** Public `std/*.cp` constants the prelude imports (no INF sentinel; use ?Int / None). */
export const PRELUDE_CONSTANTS: readonly string[] = [
    'add', 'affine', 'assign', 'flip', 'inf', 'io',
];

/** Public `std/*.cp` constants that need a `use`. */
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
    { name: 'alt', insert: 'alt(${1:program})', detail: 'a second implementation of a fn, or of the whole `program`, that `cpl stress` and `cpl locate` compare against' },
    { name: 'tags', insert: "tags('${1:.}', '${2:#}')", detail: 'compact character-set tag for grid cells' },
    { name: 'gen', insert: 'gen', detail: 'custom whole-input generator for `cpl stress`' },
    { name: 'check', insert: 'check', detail: 'constructive or multi-answer validator for `cpl stress` and `cpl test`' },
    { name: 'interactor', insert: 'interactor', detail: 'local interactive judge taking `sol: Interactor`' },
    { name: 'grader', insert: 'grader', detail: 'IOI grader-provided function or struct' },
    { name: 'get', insert: 'get', detail: 'library getter called without parentheses' },
    { name: 'test', insert: 'test', detail: 'unit test function run by `cpl test`' },
    { name: 'law', insert: 'law ${1:disjoint}(${2:i}, ${3:j}) = ${4:i != j}', detail: 'library disjointness or algebraic law annotation' },
];
