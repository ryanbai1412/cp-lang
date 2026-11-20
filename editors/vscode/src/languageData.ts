// What the fallback completion offers. The reserved words mirror
// `crates/cpl-syntax/src/token.rs` and the prelude mirrors
// `crates/cpl-check/src/prelude.rs`; `test/unit/languageData.test.ts` fails
// when either drifts, and checks the grammar against the same lists.

export const KEYWORDS: readonly string[] = [
    'let', 'var', 'fn', 'struct', 'enum', 'domain', 'type', 'const', 'trait', 'impl', 'use', 'pub',
    'if', 'elif', 'else', 'for', 'while', 'in', 'match', 'return', 'break', 'continue',
    'and', 'or', 'not', 'assert', 'with', 'by', 'mut', 'self', 'true', 'false', 'none',
    'yield', 'where', 'unchecked',
];

/** Words that are keywords only in one position, so they stay usable as names. */
export const CONTEXTUAL_KEYWORDS: readonly string[] = [
    'as', 'do', 'from', 'yields', 'law', 'lends', 'spec', 'requires', 'ensures', 'invariant',
    'static', 'inline', 'decorator', 'protocol', 'defer',
];

export const PRELUDE_VALUES: readonly string[] = [
    'INF', 'print', 'read', 'min', 'max', 'abs', 'sum', 'count', 'all', 'any', 'find', 'first',
    'last', 'argmax', 'argmin',
];

export const PRELUDE_TYPES: readonly string[] = [
    'Int', 'Float', 'Bool', 'Char', 'Str', 'I32', 'U8', 'U64', 'I128', 'List', 'Map', 'Set',
];

export interface DecoratorInfo {
    name: string;
    /** Text inserted after `@`, as a snippet. */
    insert: string;
    detail: string;
}

export const DECORATORS: readonly DecoratorInfo[] = [
    { name: 'memo', insert: 'memo', detail: 'cache the fn on its arguments' },
    { name: 'multitest', insert: 'multitest', detail: 'run the file once per test case' },
    { name: 'brute', insert: 'brute(${1:slow})', detail: 'pair this fn with a slow one for `cpl stress`' },
    { name: 'sample', insert: 'sample("${1:input}", "${2:output}")', detail: 'an inline sample' },
    { name: 'test', insert: 'test', detail: 'a test fn, run by `cpl test`' },
    { name: 'get', insert: 'get', detail: 'call this method without parentheses' },
    { name: 'invariant', insert: 'invariant(${1:rev})', detail: 'a symmetry the answer must not change under' },
    { name: 'local', insert: 'local', detail: 'exists only in local builds' },
    { name: 'derive', insert: 'derive(${1:Gen})', detail: 'generate an impl' },
    { name: 'interactor', insert: 'interactor(${1:Protocol})', detail: "the judge's side of a protocol" },
    { name: 'checker', insert: 'checker', detail: 'accepts any valid answer' },
];
