// The extension's word lists and TextMate grammar must match what CP Lang
// 0.2.2 really ships: the compiler's prelude, `stdlib/*.cp` and
// `stdlib/prelude.cp`, with zero stale v1/v2.1 tokens.

import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { test } from 'node:test';
import {
    CONTESTANT_KEYWORDS,
    CONTEXTUAL_KEYWORDS,
    DECORATORS,
    FILE_DIRECTIVES,
    KEYWORDS,
    LIBRARY_KEYWORDS,
    PRELUDE_CONSTANTS,
    STDLIB_CONSTANTS,
    PRELUDE_TYPES,
    PRELUDE_VALUES,
    STDLIB_TYPES,
    STDLIB_VALUES,
} from '../../src/languageData';
import { GRAMMAR_PATH, REPO_ROOT, scopesOf } from './grammar';

const grammarText = fs.readFileSync(GRAMMAR_PATH, 'utf8');

function sorted(names: Iterable<string>): string[] {
    return [...names].sort();
}

const STALE_V1_KEYWORDS = [
    'by', 'with', 'static', 'protocol', 'decorator',
    'requires', 'ensures', 'pure', 'undo', 'loop', 'pool', 'step', 'into',
];

const REMOVED_V21_KEYWORDS = ['defer', 'assume', 'where', 'inline', 'lend'];

const STALE_V1_TYPES = ['Float', 'I32', 'U8', 'I64', 'U32', 'ModR'];

const REMOVED_V21_BUILTINS = [
    'bsearch_first_some', 'slide_until',
    'topo_prefix',
];

const STALE_V1_DECORATORS = ['local', 'derive', 'once', 'wire', 'invariant', 'sample'];

test('keyword budget matches CP Lang 0.2.2 (30 contestant + 4 library = 34)', () => {
    assert.equal(CONTESTANT_KEYWORDS.length, 30);
    assert.equal(LIBRARY_KEYWORDS.length, 4);
    assert.equal(KEYWORDS.length, 34);
    assert.equal(new Set(KEYWORDS).size, 34);
    for (const stale of [...STALE_V1_KEYWORDS, ...REMOVED_V21_KEYWORDS]) {
        assert.ok(!KEYWORDS.includes(stale), `stale/removed keyword in KEYWORDS: ${stale}`);
        assert.ok(!CONTEXTUAL_KEYWORDS.includes(stale), `stale/removed keyword in CONTEXTUAL_KEYWORDS: ${stale}`);
    }
});

test('contextual keywords are disjoint from reserved keywords', () => {
    const reserved = new Set(KEYWORDS);
    assert.deepEqual(CONTEXTUAL_KEYWORDS.filter(word => reserved.has(word)), []);
});

/** The string literals from `from` to the next `end` in the compiler's prelude. */
function rustStrings(source: string, from: RegExp, end: string): string[] {
    const start = source.search(from);
    assert.ok(start >= 0, `no ${from} in crates/cpl-check/src/prelude.rs`);
    const body = source.slice(start, source.indexOf(end, start));
    return [...body.matchAll(/"([A-Za-z_][A-Za-z0-9_]*)"/g)].map(m => m[1]);
}

/** What CP Lang 0.2.2 really ships, read from the compiler and the standard library. */
function shipped() {
    const prelude = fs.readFileSync(path.join(REPO_ROOT, 'crates', 'cpl-check', 'src', 'prelude.rs'), 'utf8');
    const typeNames = rustStrings(prelude, /const TYPE_NAMES: &\[&str\] = &\[/, '];');
    const builtinValues = new Set([
        ...rustStrings(prelude, /pub fn value\(/, '\n}\n'),
        ...rustStrings(prelude, /pub const VALUE_NAMES: &\[&str\] = &\[/, '];'),
        ...typeNames,
    ]);
    const builtinTypes = new Set([...rustStrings(prelude, /pub fn ty\(/, '\n}\n'), ...typeNames]);
    const stdlib = path.join(REPO_ROOT, 'stdlib');
    const imported = new Set<string>();
    for (const use of fs.readFileSync(path.join(stdlib, 'prelude.cp'), 'utf8').matchAll(/^use std\.\w+\.\{([^}]*)\}/gm)) {
        for (const name of use[1].split(',').map(name => name.trim()).filter(Boolean)) {
            imported.add(name);
        }
    }
    // A `@get` fn is read as a field (`'a'.upper`, `r.l`), so it is a name of its own only when
    // the prelude imports it.
    const declared = new Map<string, string>();
    for (const file of fs.readdirSync(stdlib).filter(name => name.endsWith('.cp'))) {
        for (const line of fs.readFileSync(path.join(stdlib, file), 'utf8').split('\n')) {
            const decl = /^pub (@get )?(?:@\w+ )*(fn|struct|enum|trait|type|const|domain) ([A-Za-z_][A-Za-z0-9_]*)/.exec(line);
            // `<fn>_of_pairs`/`<fn>_of_triples` are what `T.fn(list)` resolves to, not names to type.
            // A `_` name is the stdlib's own helper, not one a contestant types.
            if (decl && !decl[3].startsWith('_') && (!decl[1] || imported.has(decl[3])) && !/_of_(pairs|triples)$/.test(decl[3])) {
                declared.set(decl[3], decl[2]);
            }
        }
    }
    const kinds = (...wanted: string[]) =>
        [...declared].filter(([, kind]) => wanted.includes(kind)).map(([name]) => name);
    return {
        builtinValues,
        builtinTypes,
        imported,
        stdlibFns: kinds('fn'),
        stdlibTypes: kinds('struct', 'enum', 'trait', 'type', 'domain'),
        stdlibConsts: kinds('const'),
    };
}

/** Names the compiler's prelude knows that the tables leave out on purpose. */
const UNLISTED_BUILTINS: Record<string, string> = {
    INF: 'no INF sentinel; use ?Int / none',
    Opt: 'spelled ?T',
    Counter: 'only the result of counts(); cpl run rejects it as a written type',
    argmax: 'checked, but cpl run has no lowering yet',
    argmin: 'checked, but cpl run has no lowering yet',
    columns: 'checked, but cpl run has no lowering yet',
    last: 'checked, but cpl run has no lowering yet',
    take: 'checked as a builtin, but cpl run has no lowering yet (std.seq.take needs a use)',
};

test('every table entry is a name CP Lang 0.2.2 ships: no phantom symbols', () => {
    const real = shipped();
    const phantoms = (names: readonly string[], known: (name: string) => boolean) => names.filter(name => !known(name));
    const fromPrelude = (name: string) => real.imported.has(name);
    assert.deepEqual(phantoms(PRELUDE_TYPES, name => real.builtinTypes.has(name)), [], 'PRELUDE_TYPES not in prelude.rs');
    assert.deepEqual(phantoms(STDLIB_TYPES, name => real.stdlibTypes.includes(name)), [], 'STDLIB_TYPES not declared in stdlib/');
    assert.deepEqual(
        phantoms(PRELUDE_VALUES, name => real.builtinValues.has(name) || (fromPrelude(name) && real.stdlibFns.includes(name))),
        [],
        'PRELUDE_VALUES neither built in nor imported by stdlib/prelude.cp',
    );
    assert.deepEqual(
        phantoms(STDLIB_VALUES, name => real.stdlibFns.includes(name) && !fromPrelude(name)),
        [],
        'STDLIB_VALUES not a stdlib fn that needs a `use`',
    );
    assert.deepEqual(
        phantoms(PRELUDE_CONSTANTS, name => real.stdlibConsts.includes(name) && fromPrelude(name)),
        [],
        'PRELUDE_CONSTANTS not a stdlib const imported by stdlib/prelude.cp',
    );
    assert.deepEqual(
        phantoms(STDLIB_CONSTANTS, name => real.stdlibConsts.includes(name) && !fromPrelude(name)),
        [],
        'STDLIB_CONSTANTS not a stdlib const that needs a `use`',
    );
    for (const name of [...PRELUDE_TYPES, ...PRELUDE_VALUES]) {
        assert.ok(!(name in UNLISTED_BUILTINS), `${name} is listed, but ${UNLISTED_BUILTINS[name]}`);
    }
});

test('every name CP Lang 0.2.2 ships is in a table: no missing symbols', () => {
    const real = shipped();
    const listed = new Set([...PRELUDE_TYPES, ...STDLIB_TYPES, ...PRELUDE_VALUES, ...STDLIB_VALUES, ...PRELUDE_CONSTANTS, ...STDLIB_CONSTANTS]);
    const builtins = [...real.builtinTypes, ...real.builtinValues].filter(name => !(name in UNLISTED_BUILTINS));
    assert.deepEqual(builtins.filter(name => !listed.has(name)), [], 'prelude.rs names missing from the tables');
    const stdlib = [...real.stdlibFns, ...real.stdlibTypes, ...real.stdlibConsts, ...real.imported];
    assert.deepEqual(stdlib.filter(name => !listed.has(name)), [], 'stdlib/ names missing from the tables');
    for (const name of Object.keys(UNLISTED_BUILTINS)) {
        assert.ok(real.builtinValues.has(name) || real.builtinTypes.has(name), `UNLISTED_BUILTINS has ${name}, which prelude.rs no longer knows`);
    }
});

test('stale v1 and removed v2.1 names stay out of the tables; decorators match 0.2.2', () => {
    for (const stale of STALE_V1_TYPES) {
        assert.ok(!PRELUDE_TYPES.includes(stale), `stale v1 type in PRELUDE_TYPES: ${stale}`);
        assert.ok(!STDLIB_TYPES.includes(stale), `stale v1 type in STDLIB_TYPES: ${stale}`);
    }
    for (const removed of REMOVED_V21_BUILTINS) {
        assert.ok(!PRELUDE_VALUES.includes(removed), `removed v2.1 builtin in PRELUDE_VALUES: ${removed}`);
        assert.ok(!STDLIB_VALUES.includes(removed), `removed v2.1 builtin in STDLIB_VALUES: ${removed}`);
    }
    assert.deepEqual(
        sorted(FILE_DIRECTIVES.map(d => d.name)),
        ['interactive', 'multitest'],
    );
    const decoratorNames = DECORATORS.map(d => d.name);
    assert.deepEqual(
        sorted(decoratorNames),
        sorted(['memo', 'multitest', 'brute', 'tags', 'gen', 'checker', 'interactor', 'grader', 'get', 'test', 'law']),
    );
    for (const stale of STALE_V1_DECORATORS) {
        assert.ok(!decoratorNames.includes(stale), `stale v1 decorator: @${stale}`);
    }
});

test('the grammar lists the exact same prelude and stdlib tables as languageData.ts', () => {
    const alternation = (scope: string) => {
        const rule = new RegExp(`"name": "${scope}",\\s*"match": "[^"]*?\\(([A-Za-z0-9_|]+)\\)`).exec(grammarText);
        assert.ok(rule, `no rule for ${scope}`);
        return rule[1].split('|');
    };
    assert.deepEqual(sorted(alternation('support.type.builtin.cp')), sorted(PRELUDE_TYPES));
    assert.deepEqual(sorted(alternation('support.type.stdlib.cp')), sorted(STDLIB_TYPES));
    assert.deepEqual(sorted(alternation('support.function.builtin.cp')), sorted(PRELUDE_VALUES));
    assert.deepEqual(sorted(alternation('support.function.stdlib.cp')), sorted(STDLIB_VALUES));
    assert.deepEqual(sorted(alternation('support.constant.stdlib.cp')), sorted([...PRELUDE_CONSTANTS, ...STDLIB_CONSTANTS]));
});

test('the grammar gives every 0.2.2 reserved word a keyword/storage/literal scope', async () => {
    for (const word of KEYWORDS) {
        const scopes = await scopesOf(`x ${word} y`, word);
        assert.ok(
            scopes.some(scope => /^(keyword|storage|constant\.language|variable\.language)\./.test(scope)),
            `\`${word}\` is scoped ${JSON.stringify(scopes)}`,
        );
    }
});

test('contextual keywords are plain names where they are not keywords', async () => {
    for (const word of CONTEXTUAL_KEYWORDS) {
        const scopes = await scopesOf(`let total = ${word} + 1`, word);
        assert.deepEqual(scopes, [], `\`${word}\` as a name is scoped ${JSON.stringify(scopes)}`);
    }
});

test('contextual keywords are keywords in their one position', async () => {
    const cases: [string, string, string][] = [
        ['    law assoc(a: T, b: T) = op(a, b) == op(b, a)', 'law', 'keyword.other.law.cp'],
        ['fn walk(self) -> yields (v: V, d: Int):', 'yields', 'keyword.other.yields.cp'],
        ['let ans: ?Int = do:', 'do', 'keyword.control.do.cp'],
    ];
    for (const [line, word, scope] of cases) {
        const scopes = await scopesOf(line, word);
        assert.ok(scopes.includes(scope), `\`${word}\` in \`${line}\` is scoped ${JSON.stringify(scopes)}`);
    }
});

test('stale v1 and removed v2.1 tokens do not get keyword, builtin or deprecated scopes in the grammar', async () => {
    for (const word of [...STALE_V1_KEYWORDS, ...REMOVED_V21_KEYWORDS]) {
        const scopes = await scopesOf(`let x = ${word} + 1`, word);
        assert.deepEqual(scopes, [], `stale/removed word \`${word}\` got scopes ${JSON.stringify(scopes)}`);
    }
    for (const ty of STALE_V1_TYPES) {
        const scopes = await scopesOf(`let x: ${ty} = y`, ty);
        assert.ok(
            !scopes.some(scope => scope.startsWith('support.type.')),
            `stale v1 type \`${ty}\` got builtin scope ${JSON.stringify(scopes)}`,
        );
    }
    const infScopes = await scopesOf('let x = INF', 'INF');
    assert.ok(
        !infScopes.some(scope => scope.startsWith('support.constant.')),
        `stale INF got builtin constant scope ${JSON.stringify(infScopes)}`,
    );
});

test('file-level directives (@!...) are scoped distinctly from item attributes (@...)', async () => {
    const directiveScopes = await scopesOf('@!multitest', 'multitest');
    assert.ok(directiveScopes.includes('keyword.control.directive.cp'), `got ${JSON.stringify(directiveScopes)}`);
    assert.ok(!directiveScopes.includes('entity.name.function.decorator.cp'), `got ${JSON.stringify(directiveScopes)}`);
    const decoratorScopes = await scopesOf('@memo', 'memo');
    assert.ok(decoratorScopes.includes('entity.name.function.decorator.cp'), `got ${JSON.stringify(decoratorScopes)}`);
    assert.ok(!decoratorScopes.includes('keyword.control.directive.cp'), `got ${JSON.stringify(decoratorScopes)}`);
});
