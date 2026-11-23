// Runs the grammar over every `.cp` file in `corpus/problems/`, `crates/cpl-cli/tests/cases/`,
// `std/` and `examples/local-lib/`: real programs, not lines written to
// suit the grammar.

import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { test } from 'node:test';
import { REPO_ROOT, Token, tokenize } from './grammar';

function cpFiles(dir: string): string[] {
    if (!fs.existsSync(dir)) {
        return [];
    }
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

const VALID_NUMBER = /^(?:0[xX][0-9a-fA-F_]+|0[bB][01_]+|[0-9][0-9_]*(?:\.[0-9][0-9_]*)?(?:[eE][+-]?[0-9][0-9_]*)?)$/;

test('the grammar cleanly tokenizes every .cp program in corpus/problems/, cases/, std/ and examples/', async () => {
    const files = [
        ...cpFiles(path.join(REPO_ROOT, 'corpus', 'problems')),
        ...cpFiles(path.join(REPO_ROOT, 'crates', 'cpl-cli', 'tests', 'cases')),
        ...cpFiles(path.join(REPO_ROOT, 'std')),
        ...cpFiles(path.join(REPO_ROOT, 'examples', 'local-lib')),
    ];
    assert.ok(files.length >= 50, `only ${files.length} .cp files found`);
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
            for (const [at, token] of row.entries()) {
                assert.ok(!has(token, 'invalid.'), `invalid scope on \`${token.text}\`: ${where(index)}`);
                if (!plain) {
                    continue;
                }
                if (VALID_NUMBER.test(token.text)) {
                    assert.ok(has(token, 'constant.numeric.'), `number \`${token.text}\`: ${where(index)}`);
                }
                const keyword = /^(let|var|fn|for|while|if|elif|else|return|match|struct|enum|domain)$/.test(token.text);
                const member = at > 0 && row[at - 1].text === '.';
                if (keyword && member) {
                    assert.ok(has(token, 'entity.name.function.method.') || has(token, 'variable.other.property.'), `member \`${token.text}\`: ${where(index)}`);
                } else if (keyword && !new RegExp(`[(,]\\s*${token.text}:`).test(line)) {
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
