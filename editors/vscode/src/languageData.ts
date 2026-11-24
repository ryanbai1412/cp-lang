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
    'ByAngleFrom', 'Centroids', 'Circle', 'ColorMatroid', 'Comb', 'Comm', 'Commutative',
    'Complement', 'Compressed', 'Concave', 'ConnectedGraph', 'Convex', 'CostFlow', 'Counter',
    'Curved', 'Dag', 'Deque', 'Desc', 'Diff', 'Digraph', 'Dir4', 'Dir8', 'DirichletSum', 'Disjoint',
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
    'MergeSegs', 'MergeSortTree', 'MergeTree', 'MergeTrees', 'MinCount', 'MinHeap', 'MinMaxHeap',
    'Mod', 'ModComb', 'MongePath', 'MonoStack', 'Monoid', 'MonotoneHull', 'Next', 'OnlineHull',
    'OnlineLis', 'Only', 'Op', 'Ordering', 'PArena', 'PMultiset', 'Pair', 'Palindromes',
    'PalindromicTree', 'ParityDsu', 'Paths', 'Perm', 'PermKind', 'PermNode', 'PermTree',
    'PersistentArray', 'PersistentDsu', 'PersistentLazySegTree', 'PersistentSegTree', 'Plane3',
    'PointSet', 'Poly', 'PotentialDsu', 'PreferredPaths', 'Prefix', 'PrefixXorBasis',
    'PrimePowerComb', 'Pt', 'Pt3', 'RangeFenwick', 'RangeKth', 'Ratio', 'Rebased', 'Rect',
    'RelaxedConvolve', 'Rev', 'Reversible', 'Reweighted', 'Ring', 'RingRange', 'RingSlice', 'Rng',
    'RollbackArray', 'RollingHash', 'RollingHash61', 'Rooted', 'Rope', 'SegTree', 'SegTree2D',
    'Selective', 'SemiRelaxedConvolve', 'Semigroup', 'Semiring', 'Sep', 'Seq', 'SeqFold', 'Settle',
    'Sieve', 'Sign', 'SimpleGraph', 'Sized', 'Slice', 'SortedBag', 'SortedMap', 'SortedRange',
    'SortedSet', 'SosOver', 'SparseFenwick2D', 'SparseIntervalMap', 'SparseIntervalSet',
    'SparseSet', 'SparseTable', 'SparseTable2D', 'Step', 'StepKeys', 'Stepped', 'SubproductTree',
    'SubseqStep', 'Subset', 'SuffixArray', 'SuffixAutomaton', 'SuffixTreeEdge', 'Sweep', 'SweepAdd',
    'Target', 'Ties', 'TimeStep', 'TimeTree', 'Track', 'Tracked', 'Tree', 'Trie', 'TwoSat', 'Undo',
    'Undoing', 'Vec', 'Vec3', 'Version', 'VirtualTree', 'Visit', 'Walk', 'WaveletMatrix',
    'Weighted', 'WeightedNext', 'Window', 'Wire', 'XorBasis', 'XorFenwick', 'XorTrie',
];

/**
 * Functions a program can call without a `use`: the compiler's own (`prelude.rs`) and the
 * ones `std/prelude.cp` imports.
 */
export const PRELUDE_VALUES: readonly string[] = [
    'abs', 'acos', 'adj4', 'adj8', 'adjacency', 'adjacent', 'aliens', 'all', 'and_then',
    'angle_less', 'any', 'area2', 'asin', 'ask', 'atan', 'atan2', 'backtrack', 'bellman_ford',
    'bfs', 'bfs01', 'binomial_transform', 'bit', 'bit_ceil', 'bit_len', 'bit_width', 'blocks',
    'borders', 'bridges', 'bsearch_real', 'buckets', 'cbrt', 'ceil', 'ceil_div', 'chain', 'choose',
    'choose2', 'chunks', 'circle_circle_tangents', 'circle_intersect', 'circle_line_intersect',
    'circle_tangents', 'circles_union_area', 'circulation', 'circumcenter', 'close', 'closest_pair',
    'closest_pair2', 'cmp', 'common_prefix_len', 'common_suffix_len', 'complement', 'components',
    'compress', 'condense', 'convex_hull', 'convex_hull_idx', 'convolve', 'convolve_2d', 'copy',
    'cos', 'count', 'counts', 'cross', 'crt', 'crt_all', 'ctz', 'cyclic_pairs', 'debug', 'dedup',
    'deduped', 'desc', 'dfs', 'dial', 'diameter2', 'digit', 'digits', 'dijkstra', 'dinkelbach',
    'distinct', 'divisors', 'divisors_from_factors', 'divmod', 'dot', 'drop_while', 'each',
    'ends_with', 'equal_range', 'euler_walk', 'eval', 'exp', 'ext_gcd', 'factor', 'find',
    'find_all', 'find_cycle', 'first', 'first_gt', 'fixed', 'flat_map', 'flatten', 'floor',
    'floor_quotients', 'floor_sum', 'floyd_warshall', 'flush', 'fold', 'gcd', 'gen',
    'general_matching', 'get_or_insert', 'global_min_cut', 'gomory_hu', 'group',
    'half_plane_intersect', 'hull_extreme', 'hull_tangents', 'hypot', 'insert', 'inv_mod',
    'inverse', 'inversions', 'is_none_or', 'is_palindrome', 'is_prime', 'is_some_and', 'is_sorted',
    'is_subsequence', 'isqrt', 'iterate', 'knuth', 'kruskal', 'kruskal_tree', 'last_lt',
    'laws_hold', 'lcm', 'len', 'line_dist', 'line_intersect', 'lis', 'ln', 'locate',
    'locate_convex', 'log10', 'log2', 'lower', 'lower_hull', 'lyndon_factorization', 'lyndon_runs',
    'manacher', 'max', 'max_bounded_flow', 'max_closure', 'max_matching',
    'max_weight_general_matching', 'max_weight_matching', 'min', 'min_bounded_flow',
    'min_bounding_box', 'min_rotation', 'min_vertex_cover', 'min_weight_matching', 'minkowski_sum',
    'minkowski_upper', 'minmax', 'mo', 'mod_log', 'mod_sqrt', 'monge_path', 'monotone_minima',
    'mul_mod', 'next_permutation', 'ntt', 'on_segment', 'overlap', 'pairs', 'parallel_bsearch',
    'partition', 'peel', 'period', 'phi', 'polygon_area', 'polygon_cut', 'popcount', 'pow',
    'pow_mod', 'prefix', 'prefix_function', 'prev_permutation', 'primitive_root', 'print', 'prod',
    'product', 'project', 'rand', 'ray', 'reachable', 'read', 'read_all', 'read_line',
    'read_output', 'reduce', 'reflect', 'remove', 'repeat', 'reverse', 'rotate_left',
    'rotate_right', 'rotated_left', 'rotated_right', 'round', 'runs', 'same_dir', 'scan', 'scc',
    'seg_dist', 'segments_meet', 'select', 'shuffle', 'sieve_multiplicative', 'sign', 'sign_eps',
    'sin', 'smawk', 'some', 'sort', 'sorted', 'sos', 'span', 'split_best', 'splitmix64', 'sqrt',
    'starts_with', 'step', 'suffix_array', 'sum', 'swap', 'take_while', 'tally', 'tan',
    'ternary_max', 'ternary_max_real', 'ternary_min', 'ternary_min_real', 'to_str', 'topo', 'track',
    'trunc', 'turn', 'two_color', 'two_edge_components', 'undoing', 'upper', 'upper_hull',
    'window_fold', 'windows', 'yes', 'z_function', 'zip',
];

/** Public `std/*.cp` functions that need a `use` (`use std.strings.{z_function}`). */
export const STDLIB_VALUES: readonly string[] = [
    'adjacency', 'adjacent_diff', 'all_some', 'angle_between', 'any_segments_meet',
    'any_segments_meet_idx', 'arborescence', 'arc_len', 'area2_128', 'arg', 'argmax', 'argmin',
    'argmins', 'bell_table', 'berlekamp_massey', 'bernoulli_table', 'best_matching',
    'best_rational', 'bit_subsets', 'block_cut_tree', 'boruvka', 'bostan_mori', 'bsearch',
    'carmichael', 'cartesian_tree', 'cdq', 'ceil_sqrt', 'chirp_z', 'chromatic_number',
    'chunks_padded', 'circle_circle_area', 'circle_polygon_area', 'clip', 'clone', 'closure',
    'collect', 'combinations', 'compose', 'compose_inv', 'concavity_break', 'continued_fraction',
    'convex_hull_3d', 'convex_hull_3d_idx', 'convex_polygon_intersect', 'convex_polygons_overlap',
    'convexity_break', 'convolve_all', 'count_in', 'count_paths', 'counting_sort', 'covering',
    'cut_vertices', 'cycle', 'dag_fold', 'de_bruijn', 'decrease', 'deg', 'delaunay', 'deriv',
    'dfs_tree', 'differences', 'dirichlet_convolve', 'dirichlet_inv', 'div_mod', 'dnc_break',
    'dnc_layer', 'dominators', 'drop', 'drop_last', 'each_divisor', 'edge_coloring', 'eq_fields',
    'eval_all', 'exp', 'farthest_pair', 'filter', 'filter_map', 'find_all_wildcard', 'first_some',
    'first_true', 'fixed_cost_paths', 'flip_action', 'from_prufer', 'fwht', 'gallop_first',
    'gcd_poly', 'girth', 'guess_at', 'halve', 'hash_fields', 'hull_line_intersect', 'hull_width',
    'idempotent', 'incenter', 'increase', 'index_pairs', 'integral', 'interpolate', 'intersect',
    'inv', 'invert', 'iroot', 'is_permutation', 'jacobi', 'johnson', 'join', 'k_shortest', 'keyed',
    'knuth_break', 'kway_merge', 'lagrange_at', 'last_true', 'lcs', 'lcs_all',
    'line_plane_intersect', 'linear_congruence', 'linear_diophantine', 'lis_ends', 'log',
    'lower_hull_idx', 'lt_fields', 'majority', 'manhattan_mst', 'map', 'masks_of_size',
    'matroid_intersection', 'max', 'max_clique', 'max_count', 'max_k', 'max_rotation', 'maximal',
    'mean', 'merge_sorted', 'min', 'min_cost_circulation', 'min_count', 'min_enclosing_circle',
    'min_k', 'min_mean_cycle', 'min_mod_in_range', 'minimal', 'minkowski_lower', 'mo_order',
    'mo_order_hilbert', 'mo_rollback', 'mo_steps', 'mo_with_updates', 'mod_kth_root', 'monge_break',
    'monge_paths', 'mult_order', 'negative_cycle', 'newton', 'next', 'next_same', 'next_smaller',
    'nth', 'odd_cycle', 'offline_over', 'only', 'opt', 'outermost', 'parse', 'parse_int',
    'partial_inverse', 'partition_in_place', 'partition_table', 'peak', 'perm_cycles', 'perm_order',
    'perm_pow', 'perm_rank', 'perm_sign', 'perm_unrank', 'permutations', 'pick_boundary',
    'pick_interior', 'pieces', 'planar_faces', 'point_action', 'point_location_planar', 'poly_at',
    'poly_degree', 'polygon_centroid', 'polygon_kernel', 'polygon_triangulate',
    'polyhedron_volume6', 'pop', 'pop_if', 'pop_while', 'pow_mod_poly', 'power_sums', 'prefix_fold',
    'prefix_matches', 'present', 'prev_same', 'prev_smaller', 'prim', 'prime_count', 'primes_in',
    'prod_linear', 'prufer', 'push', 'radix_sort', 'ray_hits_segment', 'reach_sets', 'read_fields',
    'record_highs', 'rects_union_area', 'recurrence_at', 'rekey', 'replace', 'resized', 'rfind',
    'rsearch', 'sample_points_shift', 'scc_times', 'search', 'segment_intersect', 'shift_and',
    'show_fields', 'simplex', 'smallest_topo', 'sort', 'sorted', 'sorted_diff', 'sorted_intersect',
    'sorted_union', 'spanning_forest', 'spanning_tree', 'spans', 'sparse_det', 'sparse_solve',
    'sphere_dist', 'split', 'split_once', 'split_whitespace', 'sqrt', 'stable_matching',
    'stable_partition_in_place', 'stirling1_col', 'stirling1_row', 'stirling2_col', 'stirling2_row',
    'submasks', 'subsequences', 'subset_exp', 'subset_folds', 'subset_log', 'subset_sums',
    'subtract', 'suffix_tree_edges', 'sum_mobius', 'sum_multiplicative', 'sum_phi', 'supermasks',
    'swap_at', 'symmetric', 'take', 'take_all', 'take_last', 'taylor_shift', 'to_lower', 'to_upper',
    'transpose', 'transposed', 'transposed_mul', 'triangles', 'trim', 'trim_end', 'trim_start',
    'turn_eps', 'unimodality_break', 'union', 'unzip', 'update', 'upper_hull_idx', 'valley',
    'valleys', 'voronoi_cells', 'with_domain', 'with_next', 'zip_exact',
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
