// The class names one editor grammar gives each token start the test asks about
// (crates/cpl-cli/tests/highlight_agrees.rs): `textmate` runs VS Code's generated
// grammar through vscode-textmate, `jetbrains` the plugin's editors/lexer.tsv.
//   node agree.mjs <textmate | jetbrains> QUERIES.json OUT.json
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const [engine, queriesPath, outPath] = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..', '..');
const read = file => fs.readFileSync(file, 'utf8');
const queries = JSON.parse(read(queriesPath));

async function textmate() {
    const require = createRequire(path.join(root, 'editors', 'vscode', 'package.json'));
    const tm = require('vscode-textmate');
    const onig = require('vscode-oniguruma');
    const wasm = fs.readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm'));
    await onig.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
    const file = path.join(root, 'editors', 'vscode', 'syntaxes', 'cp.tmLanguage.json');
    const registry = new tm.Registry({
        onigLib: Promise.resolve({
            createOnigScanner: sources => new onig.OnigScanner(sources),
            createOnigString: text => new onig.OnigString(text),
        }),
        loadGrammar: async () => tm.parseRawGrammar(read(file), file),
    });
    const grammar = await registry.loadGrammar('source.cp');
    return (text, at) => {
        let stack = tm.INITIAL;
        const lines = text.split('\n').map(line => {
            const r = grammar.tokenizeLine(line, stack);
            stack = r.ruleStack;
            return r.tokens;
        });
        return at.map(([line, , col]) => {
            const token = lines[line].findLast(t => t.startIndex <= col);
            return token ? token.scopes.join(' ') : '';
        });
    };
}

function jetbrains() {
    const rules = read(path.join(root, 'editors', 'lexer.tsv')).split('\n')
        .filter(line => line && !line.startsWith('#'))
        .map(line => {
            const fields = line.split('\t');
            return [fields[0], new RegExp(fields.slice(4).join('\t').replace('(?s).', '[\\s\\S]'), 'uy')];
        });
    return (text, at) => {
        const tokens = [];
        for (let pos = 0; pos < text.length;) {
            const hit = rules.find(([, re]) => { re.lastIndex = pos; return re.test(text) && re.lastIndex > pos; });
            const end = hit ? hit[1].lastIndex : pos + 1;
            tokens.push([pos, hit ? hit[0] : 'BAD_CHARACTER']);
            pos = end;
        }
        return at.map(q => tokens.findLast(([start]) => start <= q[4])?.[1] ?? '');
    };
}

const classify = engine === 'textmate' ? await textmate() : jetbrains();
fs.writeFileSync(outPath, JSON.stringify(queries.map(q => classify(read(q.path), q.at))));
