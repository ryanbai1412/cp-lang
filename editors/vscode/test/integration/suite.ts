// Runs in the extension host. Each test drives the extension through the
// same commands the editor uses, against the real `cpl lsp`.

import * as assert from 'node:assert/strict';
import * as path from 'node:path';
import * as vscode from 'vscode';

const folder = () => vscode.workspace.workspaceFolders![0].uri;
const file = (name: string) => vscode.Uri.joinPath(folder(), name);

async function until<T>(what: string, probe: () => T | undefined | Promise<T | undefined>, timeoutMs = 15000): Promise<T> {
    const deadline = Date.now() + timeoutMs;
    for (;;) {
        const value = await probe();
        if (value !== undefined) {
            return value;
        }
        if (Date.now() > deadline) {
            throw new Error(`timed out waiting for ${what}`);
        }
        await new Promise(resolve => setTimeout(resolve, 50));
    }
}

async function open(name: string): Promise<vscode.TextDocument> {
    const document = await vscode.workspace.openTextDocument(file(name));
    await vscode.window.showTextDocument(document);
    return document;
}

function hoverText(hovers: vscode.Hover[]): string {
    return hovers
        .flatMap(hover => hover.contents)
        .map(content => (typeof content === 'string' ? content : content.value))
        .join('\n');
}

const tests: [string, () => Promise<void>][] = [
    [
        'a .cp file is a cp document and starts the server',
        async () => {
            const document = await open('d.cp');
            assert.equal(document.languageId, 'cp');
            // The server answers hover once it is up; a clean file has no diagnostics.
            const text = await until('hover from cpl lsp', async () => {
                const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
                    'vscode.executeHoverProvider',
                    document.uri,
                    new vscode.Position(5, 5),
                );
                return hovers.length > 0 ? hoverText(hovers) : undefined;
            });
            assert.match(text, /```cp\npre: \[I\.cuts\]Int/);
            assert.deepEqual(vscode.languages.getDiagnostics(document.uri), []);
        },
    ],
    [
        'diagnostics arrive with their code, and the quick fix repairs the file',
        async () => {
            const document = await open('broken.cp');
            const [diagnostic] = await until('diagnostics', () => {
                const found = vscode.languages.getDiagnostics(document.uri);
                return found.length > 0 ? found : undefined;
            });
            assert.equal(diagnostic.code, 'E0101');
            assert.equal(diagnostic.source, 'cpl');
            assert.equal(diagnostic.severity, vscode.DiagnosticSeverity.Error);
            assert.equal(diagnostic.range.start.line, 3);
            assert.match(diagnostic.message, /never closed/);
            assert.equal(diagnostic.relatedInformation?.[0].message, 'a new statement starts here');

            const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
                'vscode.executeCodeActionProvider',
                document.uri,
                diagnostic.range,
                vscode.CodeActionKind.QuickFix.value,
            );
            const fix = actions.find(action => action.title === 'insert `)`');
            assert.ok(fix?.edit, `no quick fix among ${actions.map(action => action.title).join(', ')}`);
            assert.ok(await vscode.workspace.applyEdit(fix.edit));
            assert.equal(document.lineAt(3).text, 'let s = sum(dist[v] for v in V)');
            await until('the diagnostic to clear', () =>
                vscode.languages.getDiagnostics(document.uri).length === 0 ? true : undefined,
            );
        },
    ],
    [
        'go to definition and the outline come from the server',
        async () => {
            const document = await open('d.cp');
            const use = document.positionAt(document.getText().indexOf('pre[hi.after]'));
            const locations = await vscode.commands.executeCommand<vscode.Location[]>(
                'vscode.executeDefinitionProvider',
                document.uri,
                use,
            );
            assert.equal(locations[0].range.start.line, 5);
            const symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>(
                'vscode.executeDocumentSymbolProvider',
                document.uri,
            );
            const names = symbols.map(symbol => symbol.name);
            for (const name of ['I', 'pre', 'count', 'count_brute', 'answer']) {
                assert.ok(names.includes(name), `no \`${name}\` in the outline: ${names.join(', ')}`);
            }
        },
    ],
    [
        'completion offers keywords, the prelude and decorators',
        async () => {
            const document = await open('d.cp');
            const editor = vscode.window.activeTextEditor!;
            const end = document.lineAt(document.lineCount - 1).range.end;
            await editor.edit(edit => edit.insert(end, '\n@\nd'));
            const labels = async (line: number, character: number) => {
                const list = await vscode.commands.executeCommand<vscode.CompletionList>(
                    'vscode.executeCompletionItemProvider',
                    document.uri,
                    new vscode.Position(line, character),
                );
                return list.items.map(item => (typeof item.label === 'string' ? item.label : item.label.label));
            };
            const decorators = await labels(document.lineCount - 2, 1);
            assert.ok(
                decorators.includes('@memo') &&
                    decorators.includes('@brute') &&
                    decorators.includes('@grader') &&
                    decorators.includes('@checker') &&
                    decorators.includes('@interactor') &&
                    decorators.includes('@law'),
                decorators.join(', '),
            );
            assert.ok(!decorators.includes('domain'));
            const words = await labels(document.lineCount - 1, 1);
            // Only names in scope: `prev_permutation` needs a `use`, so the server leaves it out.
            for (const word of ['domain', 'unchecked', 'lends', 'assert', 'print', 'bsearch_first', 'Int', 'F64', 'Interactor', 'Mod', 'SegTree', 'Rooted']) {
                assert.ok(words.includes(word), `no \`${word}\` among the completions`);
            }
            for (const stale of ['INF', 'Float', 'I32', 'U8', 'ModR', 'defer', 'assume', 'where', 'inline', 'lend']) {
                assert.ok(!words.includes(stale), `stale/removed token \`${stale}\` appeared in completions`);
            }
            await vscode.commands.executeCommand('workbench.action.files.revert');
        },
    ],
    [
        'inlay hints annotate declared domain loops without duplicating server capabilities',
        async () => {
            const document = await open('d.cp');
            const range = new vscode.Range(0, 0, document.lineCount - 1, 0);
            const hints = await vscode.commands.executeCommand<vscode.InlayHint[]>(
                'vscode.executeInlayHintProvider',
                document.uri,
                range,
            );
            assert.ok(
                hints.some(hint => (typeof hint.label === 'string' ? hint.label : '') === ': I'),
                `expected domain loop inlay hint \`: I\`, got ${JSON.stringify(hints.map(h => h.label))}`,
            );
        },
    ],
    [
        'lenses sit above the file, its samples and its bruted part',
        async () => {
            const document = await open('d.cp');
            const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>(
                'vscode.executeCodeLensProvider',
                document.uri,
            );
            assert.deepEqual(
                lenses.map(lens => [lens.range.start.line, lens.command?.title, lens.command?.command]),
                [
                    [0, 'Memory: bounds allow 3.2 MB of 256 MB on generic', 'cpl.exec'],
                    [0, '$(play) Run on i1', 'cp.runSample'],
                    [0, 'Run on all 2 samples', 'cp.runAllSamples'],
                    [0, 'Stress', 'cp.stress'],
                    [0, 'Trace', 'cp.trace'],
                    [8, 'Stress against count_brute', 'cp.stress'],
                    [8, 'Trace', 'cp.trace'],
                ],
            );
        },
    ],
    [
        "the server's memory lens shows next to the extension's own",
        async () => {
            const document = await open('memory.cp');
            const lenses = await until('the memory lens', async () => {
                const all = await vscode.commands.executeCommand<vscode.CodeLens[]>(
                    'vscode.executeCodeLensProvider',
                    document.uri,
                );
                return all.some(lens => lens.command?.command === 'cpl.exec') ? all : undefined;
            });
            const memory = lenses.find(lens => lens.command?.command === 'cpl.exec')!.command!;
            assert.equal(memory.title, 'Memory: bounds allow 1.6 MB of 256 MB on codeforces');
            assert.deepEqual(memory.arguments, ['judges', document.uri.fsPath]);
            assert.ok((await vscode.commands.getCommands(true)).includes('cpl.exec'));
        },
    ],
    [
        'run, stress and trace finish cleanly while cpl has no back end',
        async () => {
            const document = await open('d.cp');
            // The real `cpl run` exits 3 ("not built yet"); the commands must report that, not throw.
            await vscode.commands.executeCommand('cp.runAllSamples', document.uri);
            await vscode.commands.executeCommand('cp.stress', document.uri, 'count');
        },
    ],
    [
        'run, stress and trace work against a cpl that answers them',
        async () => {
            const config = vscode.workspace.getConfiguration('cp');
            await config.update('server.path', process.env.CPL_STUB, vscode.ConfigurationTarget.Workspace);
            const document = await open('d.cp');
            const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>(
                'vscode.executeCodeLensProvider',
                document.uri,
            );
            // Run each lens exactly as a click would.
            for (const lens of lenses) {
                await vscode.commands.executeCommand(lens.command!.command, ...(lens.command!.arguments ?? []));
            }
            await until('the trace panel', () =>
                vscode.window.tabGroups.all
                    .flatMap(group => group.tabs)
                    .some(tab => tab.label === `Trace: ${path.basename(document.uri.fsPath)}`)
                    ? true
                    : undefined,
            );
            // A second trace of the same file reuses its panel.
            const trace = lenses.find(lens => lens.command?.command === 'cp.trace')!.command!;
            await vscode.commands.executeCommand(trace.command, ...(trace.arguments ?? []));
            const panels = vscode.window.tabGroups.all
                .flatMap(group => group.tabs)
                .filter(tab => tab.label === `Trace: ${path.basename(document.uri.fsPath)}`);
            assert.equal(panels.length, 1);
        },
    ],
];

export async function run(): Promise<void> {
    // The fixture workspace has no `cpl` of its own; point the extension at the one under test.
    await vscode.workspace
        .getConfiguration('cp')
        .update('server.path', process.env.CPL_REAL, vscode.ConfigurationTarget.Workspace);
    const failed: string[] = [];
    for (const [name, test] of tests) {
        try {
            await test();
            console.log(`ok - ${name}`);
        } catch (error) {
            failed.push(name);
            console.error(`not ok - ${name}\n${error instanceof Error ? error.stack : error}`);
        }
    }
    console.log(`${tests.length - failed.length} of ${tests.length} integration tests passed`);
    if (failed.length > 0) {
        throw new Error(`failed: ${failed.join('; ')}`);
    }
}
