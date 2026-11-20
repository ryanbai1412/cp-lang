// Tokenizes text with the shipped grammar, the way VS Code does.

import * as fs from 'node:fs';
import * as path from 'node:path';
import * as oniguruma from 'vscode-oniguruma';
import * as textmate from 'vscode-textmate';

/** `editors/vscode`, from `out/test/unit`. */
export const EXTENSION_ROOT = path.resolve(__dirname, '..', '..', '..');
/** The repository root. */
export const REPO_ROOT = path.resolve(EXTENSION_ROOT, '..', '..');
export const GRAMMAR_PATH = path.join(EXTENSION_ROOT, 'syntaxes', 'cp.tmLanguage.json');

export interface Token {
    text: string;
    scopes: string[];
}

let grammar: Promise<textmate.IGrammar> | undefined;

function load(): Promise<textmate.IGrammar> {
    grammar ??= (async () => {
        const wasm = fs.readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm'));
        await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
        const registry = new textmate.Registry({
            onigLib: Promise.resolve({
                createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
                createOnigString: text => new oniguruma.OnigString(text),
            }),
            loadGrammar: async scope =>
                scope === 'source.cp'
                    ? textmate.parseRawGrammar(fs.readFileSync(GRAMMAR_PATH, 'utf8'), GRAMMAR_PATH)
                    : null,
        });
        const loaded = await registry.loadGrammar('source.cp');
        if (!loaded) {
            throw new Error('the grammar did not load');
        }
        return loaded;
    })();
    return grammar;
}

/** The tokens of each line of `text`. */
export async function tokenize(text: string): Promise<Token[][]> {
    const loaded = await load();
    let stack = textmate.INITIAL;
    return text.split('\n').map(line => {
        const result = loaded.tokenizeLine(line, stack);
        stack = result.ruleStack;
        return result.tokens.map(token => ({
            text: line.slice(token.startIndex, token.endIndex),
            scopes: token.scopes,
        }));
    });
}

/** The scopes of the first token whose text is `text`, without the root scope. */
export async function scopesOf(line: string, text: string): Promise<string[]> {
    const [tokens] = await tokenize(line);
    const token = tokens.find(t => t.text.trim() === text);
    if (!token) {
        throw new Error(`no token \`${text}\` in \`${line}\`: ${tokens.map(t => JSON.stringify(t.text)).join(' ')}`);
    }
    return token.scopes.filter(scope => scope !== 'source.cp');
}
