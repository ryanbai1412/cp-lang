// The extension's word lists and grammar must match the compiler's lexer
// and prelude. These tests read the Rust sources so a new keyword there
// fails here until the editor knows it.

import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { test } from 'node:test';
import { CONTEXTUAL_KEYWORDS, KEYWORDS, PRELUDE_TYPES, PRELUDE_VALUES } from '../../src/languageData';
import { GRAMMAR_PATH, REPO_ROOT, scopesOf } from './grammar';

const tokenRs = fs.readFileSync(path.join(REPO_ROOT, 'crates/cpl-syntax/src/token.rs'), 'utf8');
const preludeRs = fs.readFileSync(path.join(REPO_ROOT, 'crates/cpl-check/src/prelude.rs'), 'utf8');
const grammarText = fs.readFileSync(GRAMMAR_PATH, 'utf8');

function compilerKeywords(): string[] {
    const body = /pub fn keyword\(text: &str\)[\s\S]*?\n {4}\}/.exec(tokenRs)?.[0] ?? '';
    return [...body.matchAll(/"([a-z]+)" =>/g)].map(m => m[1]);
}

function sorted(names: Iterable<string>): string[] {
    return [...names].sort();
}

test('the keyword list is the lexer\'s', () => {
    const fromCompiler = compilerKeywords();
    assert.ok(fromCompiler.length > 30, 'could not read the keywords out of token.rs');
    assert.deepEqual(sorted(KEYWORDS), sorted(fromCompiler));
});

test('contextual keywords are not reserved by the lexer', () => {
    const reserved = new Set(compilerKeywords());
    assert.deepEqual(CONTEXTUAL_KEYWORDS.filter(word => reserved.has(word)), []);
});

test('the prelude lists are the checker\'s', () => {
    const names = /VALUE_NAMES: &\[&str\] = &\[([\s\S]*?)\];/.exec(preludeRs)?.[1] ?? '';
    const values = [...names.matchAll(/"(\w+)"/g)].map(m => m[1]);
    assert.ok(values.length > 10, 'could not read VALUE_NAMES out of prelude.rs');
    const types = new Set(PRELUDE_TYPES);
    assert.deepEqual(sorted(values.filter(name => !types.has(name))), sorted(PRELUDE_VALUES));
    const tyBody = /pub fn ty\(name: &str\)[\s\S]*?\n\}/.exec(preludeRs)?.[0] ?? '';
    const tyNames = [...tyBody.matchAll(/"(\w+)"/g)].map(m => m[1]);
    assert.deepEqual(sorted(new Set(tyNames)), sorted(PRELUDE_TYPES));
});

test('the grammar lists the same prelude', () => {
    const alternation = (scope: string) => {
        const rule = new RegExp(`"name": "${scope}",\\s*"match": "[^"]*?\\(([A-Za-z0-9|]+)\\)`).exec(grammarText);
        assert.ok(rule, `no rule for ${scope}`);
        return rule[1].split('|');
    };
    assert.deepEqual(sorted(alternation('support.type.builtin.cp')), sorted(PRELUDE_TYPES));
    assert.deepEqual(
        sorted(alternation('support.function.builtin.cp')),
        sorted(PRELUDE_VALUES.filter(name => name !== 'INF')),
    );
});

test('the grammar gives every reserved word a keyword scope', async () => {
    for (const word of KEYWORDS) {
        const scopes = await scopesOf(`x ${word} y`, word);
        assert.ok(
            scopes.some(scope => /^(keyword|storage|constant\.language|variable\.language)\./.test(scope)),
            `\`${word}\` is scoped ${JSON.stringify(scopes)}`,
        );
    }
});

test('contextual keywords are plain names where they are not keywords', async () => {
    for (const word of CONTEXTUAL_KEYWORDS.filter(w => w !== 'lends' && w !== 'defer')) {
        const scopes = await scopesOf(`let total = ${word} + 1`, word);
        assert.deepEqual(scopes, [], `\`${word}\` as a name is scoped ${JSON.stringify(scopes)}`);
    }
});
