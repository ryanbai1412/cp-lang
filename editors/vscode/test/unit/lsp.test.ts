// Talks to the real `cpl lsp` over stdio with the messages the VS Code
// client sends, so a change on either side of the wire shows up here.
// Skipped when no `cpl` binary has been built (`cargo build -p cpl-cli`).

import * as assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { test } from 'node:test';
import * as rpc from 'vscode-jsonrpc/node';
import { resolveServer } from '../../src/cpl';
import { REPO_ROOT } from './grammar';

const binary = process.env.CPL ?? resolveServer('cpl', { pathEnv: '', workspaceFolders: [REPO_ROOT] });
const URI = 'file:///work/d.cp';

// One unclosed `(` on line 4 (0-based 3), with a fix; `dist` on line 3 is hoverable.
const SOURCE = ['let n: 1..=10 = read()', 'domain V = 1..=n', 'let dist: [V]Int = 0', 'let s = sum(dist[v] for v in V', 'print(s)', ''].join('\n');

interface Diagnostic {
    range: { start: { line: number; character: number }; end: { line: number; character: number } };
    severity: number;
    code: string;
    source: string;
    message: string;
}

test('cpl lsp serves diagnostics, hover, quick fixes, completion, references, rename and inlay hints', { skip: binary ? false : 'no cpl binary; run `cargo build -p cpl-cli`' }, async () => {
    const child = spawn(binary!, ['lsp'], { stdio: 'pipe' });
    const connection = rpc.createMessageConnection(
        new rpc.StreamMessageReader(child.stdout),
        new rpc.StreamMessageWriter(child.stdin),
    );
    const published: { uri: string; diagnostics: Diagnostic[] }[] = [];
    let wake: (() => void) | undefined;
    connection.onNotification('textDocument/publishDiagnostics', (params: { uri: string; diagnostics: Diagnostic[] }) => {
        published.push(params);
        wake?.();
    });
    const nextPublish = async (): Promise<Diagnostic[]> => {
        if (published.length === 0) {
            await new Promise<void>(resolve => (wake = resolve));
        }
        return published.shift()!.diagnostics;
    };
    // Each change publishes twice: the checker's diagnostics, then the full
    // set with the lints; the second is what the change settles on.
    const nextDiagnostics = async (): Promise<Diagnostic[]> => {
        await nextPublish();
        return nextPublish();
    };
    connection.listen();
    try {
        const init = await connection.sendRequest<{
            capabilities: Record<string, unknown>;
            serverInfo: { name: string; version: string };
        }>('initialize', { processId: process.pid, rootUri: null, capabilities: {} });
        assert.equal(init.serverInfo.name, 'cpl');
        assert.match(init.serverInfo.version, /^\d+\.\d+\.\d+/);
        // What `client.ts` relies on: with completion and inlay hints
        // advertised, the extension's fallbacks stay unregistered.
        assert.equal(init.capabilities.hoverProvider, true);
        assert.equal(init.capabilities.definitionProvider, true);
        assert.equal(init.capabilities.referencesProvider, true);
        assert.deepEqual(init.capabilities.renameProvider, { prepareProvider: true });
        assert.equal(init.capabilities.documentSymbolProvider, true);
        assert.equal(init.capabilities.inlayHintProvider, true);
        assert.deepEqual(init.capabilities.completionProvider, { triggerCharacters: ['.', '(', ',', '{'] });
        assert.deepEqual(init.capabilities.signatureHelpProvider, {
            triggerCharacters: ['(', ','],
            retriggerCharacters: [','],
        });
        assert.deepEqual(init.capabilities.codeActionProvider, { codeActionKinds: ['quickfix', 'source.fixAll.cpl', 'source.organizeImports'] });
        assert.deepEqual(init.capabilities.textDocumentSync, { openClose: true, change: 2 });
        await connection.sendNotification('initialized', {});

        await connection.sendNotification('textDocument/didOpen', {
            textDocument: { uri: URI, languageId: 'cp', version: 1, text: SOURCE },
        });
        const diagnostics = await nextDiagnostics();
        assert.equal(diagnostics.length, 1);
        const [unclosed] = diagnostics;
        assert.equal(unclosed.code, 'E0101');
        assert.equal(unclosed.source, 'cpl');
        assert.equal(unclosed.severity, 1);
        assert.deepEqual(unclosed.range.start, { line: 3, character: 11 });
        assert.match(unclosed.message, /never closed/);

        const hover = await connection.sendRequest<{ contents: { kind: string; value: string } }>('textDocument/hover', {
            textDocument: { uri: URI },
            position: { line: 2, character: 5 },
        });
        assert.equal(hover.contents.kind, 'markdown');
        // The hover is a fenced `cp` block, which the TextMate grammar colours.
        assert.match(hover.contents.value, /^```cp\ndist: \[V\]Int/);

        const actions = await connection.sendRequest<
            { title: string; kind: string; edit: { changes: Record<string, { newText: string }[]> } }[]
        >('textDocument/codeAction', {
            textDocument: { uri: URI },
            range: unclosed.range,
            context: { diagnostics },
        });
        // The likeliest place for the `)` comes first; the others are alternatives.
        // Then fix-all, which `cpl fix` makes of the file.
        const quickfixes = actions.filter(action => action.kind === 'quickfix');
        assert.ok(quickfixes.length >= 1);
        assert.ok(quickfixes.every(action => action.title.startsWith('insert `)`')));
        assert.deepEqual(
            actions.slice(quickfixes.length).map(action => action.kind),
            ['source.fixAll.cpl'],
        );
        assert.equal(actions[0].title, 'insert `)`');
        assert.deepEqual(actions[0].edit.changes[URI].map(edit => edit.newText), [')']);

        // Applying the fix as an incremental change clears the error.
        const fix = actions[0].edit.changes[URI][0] as { newText: string; range: Diagnostic['range'] };
        await connection.sendNotification('textDocument/didChange', {
            textDocument: { uri: URI, version: 2 },
            contentChanges: [{ range: fix.range, text: fix.newText }],
        });
        assert.deepEqual(await nextDiagnostics(), []);

        const symbols = await connection.sendRequest<{ name: string }[]>('textDocument/documentSymbol', {
            textDocument: { uri: URI },
        });
        assert.deepEqual(symbols.map(symbol => symbol.name), ['n', 'V', 'dist', 's']);

        // Completion inside `print(` offers the file's names with their types.
        const completion = await connection.sendRequest<{ items: { label: string; detail?: string }[] }>(
            'textDocument/completion',
            { textDocument: { uri: URI }, position: { line: 4, character: 6 } },
        );
        const dist = completion.items.find(item => item.label === 'dist');
        assert.ok(dist, 'completion offers `dist`');
        assert.match(dist.detail ?? '', /\[V\]Int/);

        // `dist` is declared on line 3 and read on line 4 (0-based 2 and 3).
        const references = await connection.sendRequest<{ range: Diagnostic['range'] }[]>('textDocument/references', {
            textDocument: { uri: URI },
            position: { line: 2, character: 5 },
            context: { includeDeclaration: true },
        });
        assert.deepEqual(references.map(ref => ref.range.start), [
            { line: 2, character: 4 },
            { line: 3, character: 12 },
        ]);
        const rename = await connection.sendRequest<{ changes: Record<string, { newText: string; range: Diagnostic['range'] }[]> }>(
            'textDocument/rename',
            { textDocument: { uri: URI }, position: { line: 3, character: 13 }, newName: 'best' },
        );
        assert.deepEqual(
            rename.changes[URI].map(edit => [edit.range.start.line, edit.newText]),
            [[2, 'best'], [3, 'best']],
        );

        const hints = await connection.sendRequest<{ label: string }[]>('textDocument/inlayHint', {
            textDocument: { uri: URI },
            range: { start: { line: 0, character: 0 }, end: { line: 5, character: 0 } },
        });
        // The loop variable `v` on line 4 is shown with its domain.
        assert.ok(hints.some(hint => hint.label === ': V'), JSON.stringify(hints));

        await connection.sendRequest('shutdown');
        const exited = new Promise<number | null>(resolve => child.on('exit', resolve));
        await connection.sendNotification('exit');
        assert.equal(await exited, 0);
    } finally {
        connection.dispose();
        child.kill();
    }
});
