// Validates `snippets/cp.json`: every required edition 0.3 contest snippet
// exists, has valid structure, contains zero stale v1/v2.1 tokens, and tokenizes
// cleanly with the TextMate grammar.

import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { test } from 'node:test';
import { tokenize } from './grammar';

interface SnippetEntry {
    prefix: string | string[];
    body: string[];
    description: string;
}

const SNIPPETS_PATH = path.resolve(__dirname, '../../../snippets/cp.json');
const snippets: Record<string, SnippetEntry> = JSON.parse(fs.readFileSync(SNIPPETS_PATH, 'utf8'));

/** Expands VS Code snippet tabstops like `${1:dp}` and `${4|a,b|}` into plain CP Lang code. */
function expandSnippet(body: readonly string[]): string {
    return body
        .join('\n')
        .replace(/\$\{\d+\|([^|,}]*)[^}]*\}/g, '$1')
        .replace(/\$\{\d+:([^}]*)\}/g, '$1')
        .replace(/\$\{\d+\}/g, '')
        .replace(/\$\d+/g, '');
}

test('snippets/cp.json includes all required edition 0.3 contest snippets', () => {
    const allPrefixes = new Set(
        Object.values(snippets).flatMap(entry => (Array.isArray(entry.prefix) ? entry.prefix : [entry.prefix])),
    );
    for (const required of [
        'memo',
        '@!multitest',
        'multitest',
        '@alt',
        'first_true',
        'compress',
        'segtree',
        'segtree-monoid',
        'Monoid',
        'lazysegtree',
        'lazysegtree-action',
        'Action',
        'rooted',
        'tropical',
        'checker',
        'interactor',
        'stress',
        'while let',
        'if let',
        'if const',
        'unchecked',
    ]) {
        assert.ok(allPrefixes.has(required), `missing required snippet prefix: ${required}`);
    }
    const allExpanded = Object.values(snippets)
        .map(entry => expandSnippet(entry.body))
        .join('\n');
    for (const requiredToken of [
        '@!multitest',
        '@memo',
        '@alt(',
        'first_true(',
        'compress(',
        'SegTree<',
        'Monoid(',
        'act: ',
        'Action(',
        'Rooted<',
        '?Int = none',
        'min=',
        '@check',
        '@interactor',
        'sol: Interactor',
        'gen:',
        'let comb = Comb<',
        'while let x = xs.pop(), x > 0:',
        'if let x = opt, x > 0:',
        'if const FAST:',
        'unchecked:',
    ]) {
        assert.ok(allExpanded.includes(requiredToken), `snippets do not cover \`${requiredToken}\``);
    }
});

test('every snippet is well-formed, free of stale v1/v2.1 tokens, and tokenizes cleanly', async () => {
    const stalePattern = /\b(Float|I32|U8|ModR|INF|protocol|static|yields|requires|ensures|defer|assume|where|inline|lend|absorb)\b|@memo\(|for\s+mut\b/;
    for (const [name, entry] of Object.entries(snippets)) {
        assert.ok(Array.isArray(entry.body) && entry.body.length > 0, `empty body in snippet "${name}"`);
        assert.ok(typeof entry.description === 'string' && entry.description.length > 0, `missing description in "${name}"`);
        const code = expandSnippet(entry.body);
        assert.ok(!stalePattern.test(code), `stale v1/v2.1 token in snippet "${name}":\n${code}`);
        const lines = await tokenize(code);
        for (const row of lines) {
            for (const token of row) {
                assert.ok(
                    !token.scopes.some(scope => scope.startsWith('invalid.')),
                    `invalid scope in snippet "${name}" on \`${token.text}\``,
                );
            }
        }
    }
});
