// CP Lang v2.1 language tables for fallback completion and grammar checks
// (`docs/one-page-cheat-sheet.md`, `research/language-architecture-v2.1.md`).
// `test/unit/languageData.test.ts` checks that these tables and
// `syntaxes/cp.tmLanguage.json` stay in lockstep with zero stale v1 tokens.

/** 32 contestant keywords in CP Lang v2.1. */
export const CONTESTANT_KEYWORDS: readonly string[] = [
    'let', 'var', 'const', 'fn', 'struct', 'enum', 'domain', 'type', 'use', 'pub',
    'if', 'elif', 'else', 'for', 'while', 'in', 'match', 'return', 'break', 'continue', 'defer',
    'and', 'or', 'not', 'assert', 'assume', 'mut', 'self', 'unchecked',
    'true', 'false', 'none',
];

/** 8 library-author keywords in CP Lang v2.1. */
export const LIBRARY_KEYWORDS: readonly string[] = [
    'trait', 'impl', 'where', 'lends', 'lend', 'law', 'inline', 'yield',
];

/** All 40 reserved keywords in CP Lang v2.1. */
export const KEYWORDS: readonly string[] = [...CONTESTANT_KEYWORDS, ...LIBRARY_KEYWORDS];

/** Words that are keywords only in one position, so they stay usable as names. */
export const CONTEXTUAL_KEYWORDS: readonly string[] = [
    'as', 'from', 'invariant',
];

/** Core primitive and container types built into CP Lang v2.1. */
export const PRELUDE_TYPES: readonly string[] = [
    'Int', 'U64', 'I128', 'F64', 'Bool', 'Char', 'Str',
    'List', 'Deque', 'Map', 'HashMap', 'Set', 'HashSet',
    'SortedBag', 'MinHeap', 'MaxHeap', 'Range', 'Bounds', 'Subset', 'Bitset', 'Iter',
];

/** Standard library contest data structures and algebra helpers in CP Lang v2.1. */
export const STDLIB_TYPES: readonly string[] = [
    'Mod', 'Comb', 'Marks', 'Compressed', 'MonoDeque',
    'SegTree', 'LazySegTree', 'SparseTable', 'Fenwick', 'Fenwick2D', 'Steps',
    'Dsu', 'Graph', 'Rooted', 'FlowGraph', 'MinCostFlow', 'TwoSat',
    'Perm', 'Sieve', 'Matrix', 'Pt', 'Dir4', 'Dir8', 'ByAngle',
    'SuffixArray', 'RollingHash', 'HashVal',
    'Monoid', 'Action', 'Desc', 'Reversible',
    'AddSum', 'AddMin', 'AddMax', 'AssignSum', 'AssignMin', 'AssignMax',
];

/** Prelude and standard-library functions highlighted as builtins in CP Lang v2.1. */
export const PRELUDE_VALUES: readonly string[] = [
    'read', 'read_line', 'read_until', 'read_output', 'print', 'debug', 'ask',
    'min', 'max', 'sum', 'prod', 'any', 'all', 'count', 'first', 'min_by', 'max_by',
    'bsearch_first', 'bsearch_last', 'bsearch_first_some',
    'sort', 'sorted', 'reverse', 'dedup', 'rotate_left', 'rotate_right', 'next_permutation',
    'compress', 'prefix', 'scan', 'runs', 'sliding_min', 'sliding_max', 'slide_until', 'swap',
    'abs', 'gcd', 'lcm', 'isqrt', 'pow_mod', 'factor', 'divisors', 'popcount', 'ctz', 'clz', 'bit_len',
    'sqrt', 'sin', 'cos', 'tan', 'atan2', 'exp', 'ln', 'hypot', 'floor', 'ceil', 'round', 'trunc',
    'close', 'sign', 'turn', 'turn_eps', 'unit', 'convex_hull', 'desc',
    'bfs', 'dijkstra', 'shortest', 'topo', 'topo_prefix', 'scc', 'condense',
    'two_edge_components', 'biconnected_components', 'articulation_points', 'two_color',
    'mst', 'max_weight_matching',
    'z_function', 'prefix_function', 'palindromes',
];

/** Built-in math constants in CP Lang v2.1. */
export const PRELUDE_CONSTANTS: readonly string[] = ['PI'];

export interface DecoratorInfo {
    name: string;
    /** Text inserted after `@`, as a snippet. */
    insert: string;
    detail: string;
}

/** The 7 contestant decorators (`checker`/`interactor` paired) + 2 library-author decorators in v2.1. */
export const DECORATORS: readonly DecoratorInfo[] = [
    { name: 'memo', insert: 'memo', detail: 'memoize a function over its domain arguments (`@memo`, `@memo(flat)`, `@memo(hash)`)' },
    { name: 'multitest', insert: 'multitest', detail: 'run the file or `solve()` once per test case' },
    { name: 'brute', insert: 'brute(${1:main})', detail: 'reference brute-force implementation for `cpl stress`' },
    { name: 'tags', insert: "tags('${1:.}', '${2:#}')", detail: 'compact character-set tag for grid cells' },
    { name: 'gen', insert: 'gen', detail: 'custom whole-input generator for `cpl stress`' },
    { name: 'checker', insert: 'checker', detail: 'constructive or multi-answer validator for `cpl stress`' },
    { name: 'interactor', insert: 'interactor', detail: 'local interactive judge for `cpl stress` and `cpl run`' },
    { name: 'grader', insert: 'grader', detail: 'IOI grader-provided constant, function, or struct' },
    { name: 'get', insert: 'get', detail: 'library getter called without parentheses' },
    { name: 'test', insert: 'test', detail: 'unit test function run by `cpl test`' },
];
