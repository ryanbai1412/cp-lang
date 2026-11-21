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
    'Range', 'Set', 'Str', 'Subset', 'U64',
];

/** Every public type and trait declared in `stdlib/*.cp`, whether `stdlib/prelude.cp` imports it or not. */
export const STDLIB_TYPES: readonly string[] = [
    'Action', 'Arc', 'Bitset', 'ByAngle', 'Comb', 'Compressed', 'Deque', 'Desc', 'Dir4', 'Dir8', 'Dsu',
    'Fenwick', 'Fenwick2D', 'FlowGraph', 'Graph', 'HashVal', 'Hashable', 'LazySegTree', 'Marks',
    'MaxHeap', 'MinHeap', 'Mod', 'Monoid', 'Neighbours', 'Prefix1', 'Prefix2', 'PrefixN', 'Pt',
    'Reversible', 'Rng', 'RollingHash', 'Rooted', 'SegTree', 'Sieve', 'SortedBag', 'SparseTable',
    'Target', 'XorFenwick',
];

/**
 * Functions a program can call without a `use`: the compiler's own (`prelude.rs`) and the
 * ones `stdlib/prelude.cp` imports.
 */
export const PRELUDE_VALUES: readonly string[] = [
    'abs', 'all', 'any', 'ask', 'atan2', 'bit_ceil', 'bit_width', 'ceil', 'copy', 'count', 'counts',
    'ctz', 'debug', 'first', 'floor', 'flush', 'isqrt', 'laws_hold', 'len', 'max', 'min',
    'popcount', 'prefix', 'print', 'rand', 'read', 'read_output', 'round', 'shuffle', 'some',
    'sort', 'sorted', 'span', 'sqrt', 'sum', 'swap', 'trunc', 'bfs', 'bit', 'bsearch_first',
    'bsearch_last', 'ceil_div', 'compress', 'condense', 'deduped', 'desc', 'digits', 'dijkstra',
    'ends_with', 'excl_fold', 'find', 'fixed', 'fold', 'gcd', 'get_or_insert', 'insert', 'lcm',
    'lower', 'max_by', 'min_by', 'partition', 'peel', 'prefix1', 'prefix2', 'prefix_n', 'remove',
    'repeat', 'resized', 'runs', 'runs_by', 'scan', 'scc', 'starts_with', 'topo', 'upper', 'digit',
    'find_all', 'step', 'windows', 'zip',
];

/** Public `stdlib/*.cp` functions that need a `use` (`use std.strings.{z_function}`). */
export const STDLIB_VALUES: readonly string[] = [
    'area2', 'argmins', 'articulation_points', 'berlekamp_massey', 'biconnected_components', 'bit_len',
    'choose', 'components', 'concavity_break', 'convexity_break', 'cross', 'crt', 'cyclic_pairs',
    'decrease', 'dedup', 'differences', 'directed_of_pairs', 'directed_of_triples', 'divisors',
    'dnc_break', 'dot', 'ext_gcd', 'factor', 'first_ge_by', 'get_or_insert_with', 'guess_at',
    'idempotent', 'increase', 'knuth_break', 'last_le_by', 'lcp_array', 'locate', 'max_weight_matching',
    'monge_break', 'mul_mod', 'newton', 'next_permutation', 'on_segment', 'period', 'poly_at',
    'poly_degree', 'prefix_function', 'prev_permutation', 'ray', 'recurrence_at', 'reverse', 'rfind',
    'rotate_left', 'segments_meet', 'splitmix64', 'suffix_array', 'swap_at', 'take', 'turn',
    'two_color', 'two_edge_components', 'undirected_of_pairs', 'undirected_of_triples',
    'unimodality_break', 'update', 'z_function',
];

/** Public `stdlib/*.cp` constants, all imported by `stdlib/prelude.cp` (no INF sentinel; use ?Int / none). */
export const PRELUDE_CONSTANTS: readonly string[] = [
    'AddMax', 'AddMin', 'AddSum', 'AssignMax', 'AssignMin', 'AssignSum',
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
