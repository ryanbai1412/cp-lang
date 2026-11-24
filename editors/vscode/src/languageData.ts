// CP Lang 0.2.2 language tables for fallback completion and grammar checks.
// `test/unit/languageData.test.ts` checks them against the compiler's prelude
// (`crates/cpl-check/src/prelude.rs`), `std/*.cp` and `std/prelude.cp`.

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

// The name tables are generated (scripts/language-data.js) at compile time.
export {
    PRELUDE_TYPES,
    STDLIB_TYPES,
    BUILTIN_VALUES,
    PRELUDE_VALUES,
    STDLIB_VALUES,
    PRELUDE_CONSTANTS,
    STDLIB_CONSTANTS,
} from './languageTables';

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
