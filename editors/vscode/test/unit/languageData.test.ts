// The extension's word lists and TextMate grammar must match the locked
// CP Lang v2.1 surface (`docs/one-page-cheat-sheet.md`,
// `research/language-architecture-v2.1.md`) with zero stale v1 tokens.

import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import { test } from 'node:test';
import {
    CONTESTANT_KEYWORDS,
    CONTEXTUAL_KEYWORDS,
    DECORATORS,
    KEYWORDS,
    LIBRARY_KEYWORDS,
    PRELUDE_CONSTANTS,
    PRELUDE_TYPES,
    PRELUDE_VALUES,
    STDLIB_TYPES,
} from '../../src/languageData';
import { GRAMMAR_PATH, scopesOf } from './grammar';

const grammarText = fs.readFileSync(GRAMMAR_PATH, 'utf8');

function sorted(names: Iterable<string>): string[] {
    return [...names].sort();
}

const STALE_V1_KEYWORDS = [
    'by', 'with', 'static', 'protocol', 'decorator', 'do', 'yields',
    'requires', 'ensures', 'pure', 'undo', 'loop', 'pool', 'step', 'into',
];

const STALE_V1_TYPES = ['Float', 'I32', 'U8', 'I64', 'U32', 'ModR'];

const STALE_V1_DECORATORS = ['local', 'derive', 'once', 'wire', 'invariant', 'sample'];

test('keyword budget matches CP Lang v2.1 (32 contestant + 8 library = 40)', () => {
    assert.equal(CONTESTANT_KEYWORDS.length, 32);
    assert.equal(LIBRARY_KEYWORDS.length, 8);
    assert.equal(KEYWORDS.length, 40);
    assert.equal(new Set(KEYWORDS).size, 40);
    for (const stale of STALE_V1_KEYWORDS) {
        assert.ok(!KEYWORDS.includes(stale), `stale v1 keyword in KEYWORDS: ${stale}`);
        assert.ok(!CONTEXTUAL_KEYWORDS.includes(stale), `stale v1 keyword in CONTEXTUAL_KEYWORDS: ${stale}`);
    }
});

test('contextual keywords are disjoint from reserved keywords', () => {
    const reserved = new Set(KEYWORDS);
    assert.deepEqual(CONTEXTUAL_KEYWORDS.filter(word => reserved.has(word)), []);
});

test('type, constant and decorator tables have CP Lang v2.1 parity and zero stale v1 entries', () => {
    for (const stale of STALE_V1_TYPES) {
        assert.ok(!PRELUDE_TYPES.includes(stale), `stale v1 type in PRELUDE_TYPES: ${stale}`);
        assert.ok(!STDLIB_TYPES.includes(stale), `stale v1 type in STDLIB_TYPES: ${stale}`);
    }
    assert.ok(!PRELUDE_VALUES.includes('INF'), 'stale INF in PRELUDE_VALUES');
    assert.ok(!PRELUDE_CONSTANTS.includes('INF'), 'stale INF in PRELUDE_CONSTANTS');
    for (const requiredType of [
        'Int', 'U64', 'I128', 'F64', 'Bool', 'Char', 'Str', 'Bitset',
    ]) {
        assert.ok(PRELUDE_TYPES.includes(requiredType), `missing builtin type ${requiredType}`);
    }
    for (const requiredStdlib of [
        'Mod', 'Comb', 'Marks', 'Compressed', 'MonoDeque',
        'SegTree', 'LazySegTree', 'SparseTable', 'Dsu', 'Rooted',
    ]) {
        assert.ok(STDLIB_TYPES.includes(requiredStdlib), `missing stdlib type ${requiredStdlib}`);
    }
    const decoratorNames = DECORATORS.map(d => d.name);
    assert.deepEqual(
        sorted(decoratorNames),
        sorted(['memo', 'multitest', 'brute', 'tags', 'gen', 'checker', 'interactor', 'grader', 'get', 'test']),
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
});

test('the grammar gives every v2.1 reserved word a keyword/storage/literal scope', async () => {
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

test('stale v1 tokens do not get keyword, builtin or deprecated scopes in the grammar', async () => {
    for (const word of STALE_V1_KEYWORDS) {
        const scopes = await scopesOf(`let x = ${word} + 1`, word);
        assert.deepEqual(scopes, [], `stale v1 word \`${word}\` got scopes ${JSON.stringify(scopes)}`);
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
