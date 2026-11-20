// CP Lang 0.2.2 language tables for fallback completion and grammar checks
// (`docs/one-page-cheat-sheet.md`, `research/language-architecture-v2.2.md`).
// `test/unit/languageData.test.ts` checks that these tables and
// `syntaxes/cp.tmLanguage.json` stay in lockstep with zero stale v1/v2.1 tokens.

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

/** Words that are keywords only in one position, so they stay usable as names. */
export const CONTEXTUAL_KEYWORDS: readonly string[] = [
    'as', 'from', 'invariant',
];

/** Core primitive, container, and runtime types built into CP Lang 0.2.2. */
export const PRELUDE_TYPES: readonly string[] = [
    'Int', 'U64', 'I128', 'F64', 'Bool', 'Char', 'Str',
    'List', 'Deque', 'Map', 'HashMap', 'Set', 'HashSet',
    'SortedBag', 'MinHeap', 'MaxHeap', 'Range', 'Bounds', 'Subset', 'Bitset', 'Iter',
    'Interactor', 'Rng',
];

/** Core standard library contest data structures and algebra helpers in CP Lang 0.2.2. */
export const STDLIB_TYPES: readonly string[] = [
    'Mod', 'Comb', 'Marks', 'Compressed',
    'SegTree', 'LazySegTree', 'SparseTable', 'Fenwick', 'Fenwick2D',
    'Dsu', 'Graph', 'Rooted', 'FlowGraph',
    'Perm', 'Sieve', 'Pt', 'Dir4', 'ByAngle',
    'SuffixArray', 'RollingHash', 'HashVal',
    'Monoid', 'Action', 'Desc',
    'AddSum', 'AddMin', 'AddMax', 'AssignSum', 'AssignMin', 'AssignMax',
];

/** Prelude and core standard-library functions highlighted as builtins in CP Lang 0.2.2. */
export const PRELUDE_VALUES: readonly string[] = [
    'read', 'read_line', 'read_until', 'read_output', 'print', 'debug', 'ask',
    'min', 'max', 'sum', 'any', 'all', 'count', 'first', 'min_by', 'max_by',
    'bsearch_first', 'bsearch_last',
    'sort', 'sorted', 'reverse', 'dedup', 'rotate_left', 'next_permutation', 'prev_permutation',
    'compress', 'prefix', 'scan', 'runs', 'swap', 'rand', 'shuffle',
    'abs', 'ceil_div', 'gcd', 'lcm', 'isqrt', 'pow_mod', 'factor', 'divisors',
    'popcount', 'ctz', 'clz', 'bit_len',
    'sqrt', 'sin', 'cos', 'tan', 'atan2', 'exp', 'ln', 'hypot', 'floor', 'ceil', 'round', 'trunc',
    'close', 'sign', 'turn', 'turn_eps', 'unit', 'desc',
    'bfs', 'dijkstra', 'topo', 'scc', 'condense',
    'two_edge_components', 'biconnected_components', 'articulation_points', 'two_color',
    'max_weight_matching',
    'z_function', 'prefix_function',
];

/** Built-in math constants in CP Lang 0.2.2 (no INF sentinel; use ?Int / none). */
export const PRELUDE_CONSTANTS: readonly string[] = ['PI'];

export interface DecoratorInfo {
    name: string;
    /** Text inserted after `@`, as a snippet. */
    insert: string;
    detail: string;
}

/** 8 contestant decorators + 3 library-author decorators in CP Lang 0.2.2. */
export const DECORATORS: readonly DecoratorInfo[] = [
    { name: 'memo', insert: 'memo', detail: 'memoize a function (auto-dispatches flat vs salted hash table)' },
    { name: 'multitest', insert: 'multitest', detail: 'run the file or `solve()` once per test case' },
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
