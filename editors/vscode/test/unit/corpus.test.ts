// Runs the grammar over every `.cp` file in the repository's prototype
// ports and tests: real programs, not lines written to suit the grammar.

import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { test } from 'node:test';
import { REPO_ROOT, Token, tokenize } from './grammar';

function cpFiles(dir: string): string[] {
    return fs.readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            return cpFiles(full);
        }
        return entry.name.endsWith('.cp') ? [full] : [];
    });
}

function has(token: Token, prefix: string): boolean {
    return token.scopes.some(scope => scope.startsWith(prefix));
}

test('the grammar reads every program in proto/', async () => {
    const files = [...cpFiles(path.join(REPO_ROOT, 'proto', 'ports')), ...cpFiles(path.join(REPO_ROOT, 'proto', 'tests'))];
    assert.ok(files.length >= 30, `only ${files.length} .cp files found`);
    for (const file of files) {
        const text = fs.readFileSync(file, 'utf8');
        const lines = text.split('\n');
        const tokens = await tokenize(text);
        const where = (index: number) => `${path.relative(REPO_ROOT, file)}:${index + 1}: ${lines[index]}`;
        lines.forEach((line, index) => {
            const row = tokens[index].filter(token => token.text.trim() !== '');
            // The checks below are about code, so skip lines that hold a string or a comment.
            const plain = !line.includes('"') && !line.includes("'") && !line.includes('//');
            if (/^\s*\/\//.test(line)) {
                assert.ok(row.every(token => has(token, 'comment.')), `comment line: ${where(index)}`);
                return;
            }
            // A string or comment must never swallow the start of a line of code.
            if (row.length > 0 && /^\s*[A-Za-z_@]/.test(line)) {
                assert.ok(!has(row[0], 'string.') && !has(row[0], 'comment.'), `starts inside a string: ${where(index)}`);
            }
            for (const token of row) {
                // `pure fn` was removed from the language, and the grammar says so as the compiler does.
                const removed = has(token, 'invalid.deprecated.pure.cp') && token.text === 'pure';
                assert.ok(removed || !has(token, 'invalid.'), `invalid scope on \`${token.text}\`: ${where(index)}`);
                if (!plain) {
                    continue;
                }
                if (/^[0-9]/.test(token.text)) {
                    assert.ok(has(token, 'constant.numeric.'), `number \`${token.text}\`: ${where(index)}`);
                }
                if (/^(let|var|fn|for|while|if|elif|else|return|match|struct|enum|domain)$/.test(token.text)) {
                    assert.ok(has(token, 'keyword.') || has(token, 'storage.'), `keyword \`${token.text}\`: ${where(index)}`);
                }
            }
            const fn = /^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/.exec(line);
            if (fn) {
                const name = row.find(token => token.text === fn[1]);
                assert.ok(name && has(name, 'entity.name.function.cp'), `fn name: ${where(index)}`);
            }
            const domain = /^domain\s+([A-Z][A-Za-z0-9_]*)/.exec(line);
            if (domain) {
                const name = row.find(token => token.text === domain[1]);
                assert.ok(name && has(name, 'entity.name.type.domain.cp'), `domain name: ${where(index)}`);
            }
        });
    }
});
