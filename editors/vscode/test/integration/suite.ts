// Runs in the extension host. Each test drives the extension through the
// same commands the editor uses, against the real `cpl lsp`.

import * as assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import * as fs from 'node:fs';
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
            const use = document.positionAt(document.getText().indexOf('pre[hi.r]'));
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
        'formatting comes from cpl fmt through the server',
        async () => {
            const document = await open('messy.cp');
            const edits = await until('format edits from cpl lsp', async () => {
                const got = await vscode.commands.executeCommand<vscode.TextEdit[]>(
                    'vscode.executeFormatDocumentProvider',
                    document.uri,
                    { tabSize: 4, insertSpaces: true },
                );
                return got && got.length > 0 ? got : undefined;
            });
            const edit = new vscode.WorkspaceEdit();
            edit.set(document.uri, edits);
            assert.ok(await vscode.workspace.applyEdit(edit));
            // A Windows checkout has CRLF line ends, which `cpl fmt` keeps.
            assert.equal(document.getText().replace(/\r\n/g, '\n'), 'let x = 1\nprint(x)\n');
        },
    ],
    [
        'add a test writes the next case and runs it',
        async () => {
            const document = await open('add.cp');
            const added = await vscode.commands.executeCommand<boolean>('cp.addTest', document.uri, '4', '8');
            const tests = file('add.tests').fsPath;
            if (!added) {
                // Say why: what `cpl test --add` itself gives, and how long it takes.
                const held = fs.existsSync(tests) ? fs.readdirSync(tests).join(', ') || 'nothing' : 'nothing yet';
                const start = Date.now();
                const direct = spawnSync(process.env.CPL_REAL!, ['test', document.uri.fsPath, '--add', '-'], {
                    cwd: path.dirname(document.uri.fsPath),
                    input: '4\n',
                    encoding: 'utf8',
                });
                assert.fail(
                    `add.tests held ${held}; \`cpl test --add -\` gives ${direct.status} in ${Date.now() - start} ms: ${direct.stdout}${direct.stderr}`,
                );
            }
            assert.equal(fs.readFileSync(file('add.tests/1.in').fsPath, 'utf8'), '4\n');
            assert.equal(fs.readFileSync(file('add.tests/1.out').fsPath, 'utf8'), '8\n');
        },
    ],
    [
        'import a problem takes what Competitive Companion sends',
        async () => {
            await vscode.workspace.getConfiguration('cp').update('companion.port', 10071, vscode.ConfigurationTarget.Global);
            const document = await open('add.cp');
            await vscode.commands.executeCommand('cp.importProblem', path.dirname(document.uri.fsPath));
            const problem = JSON.stringify({
                name: 'B. Twice',
                group: 'Codeforces Round',
                url: 'https://codeforces.com/problemset/problem/1/B',
                memoryLimit: 256,
                timeLimit: 1000,
                tests: [{ input: '2\n', output: '4\n' }],
                testType: 'single',
                input: { type: 'stdin' },
                output: { type: 'stdout' },
            });
            await until('the listener to take the problem', async () => {
                try {
                    const response = await fetch('http://127.0.0.1:10071/', { method: 'POST', body: problem });
                    return response.ok || response.status < 500 ? true : undefined;
                } catch {
                    return undefined;
                }
            });
            await until('b.cp and its sample', () => (fs.existsSync(file('b.tests/1.out').fsPath) ? true : undefined));
            assert.match(fs.readFileSync(file('b.cp').fsPath, 'utf8'), /B\. Twice/);
        },
    ],
    [
        'the Stress command saves a failing case that Run All Samples reruns',
        async () => {
            // The debugger's knapsack demo, with its planted bug: stress against its `@alt(program)`.
            const demo = path.resolve(__dirname, '../../../../../examples/debugging/knapsack.cp');
            const planted = fs.readFileSync(demo, 'utf8');
            assert.ok(planted.includes('for c in 0..=cap:'));
            fs.writeFileSync(file('k.cp').fsPath, planted);
            const document = await open('k.cp');
            const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>(
                'vscode.executeCodeLensProvider',
                document.uri,
            );
            assert.ok(lenses.some(lens => lens.command?.command === 'cp.stress'), 'no Stress lens');
            await vscode.commands.executeCommand('cp.stress', document.uri);
            const saved = await until(
                'the saved failing case',
                () => {
                    const dir = file('k.tests').fsPath;
                    const names = fs.existsSync(dir) ? fs.readdirSync(dir).filter(n => /^stress-\d+\.in$/.test(n)) : [];
                    return names.length > 0 ? names : undefined;
                },
                60000,
            );
            assert.equal(saved.length, 1);
            await vscode.commands.executeCommand('cp.runAllSamples', document.uri);
            fs.writeFileSync(file('k.cp').fsPath, planted.replace('for c in 0..=cap:', 'for c in (0..=cap).rev:'));
            await vscode.commands.executeCommand('workbench.action.files.revert');
            await vscode.commands.executeCommand('cp.runAllSamples', document.uri);
            await vscode.commands.executeCommand('cp.stress', document.uri);
        },
    ],
    [
        'the debugger steps back to where the last step came from',
        async () => {
            fs.writeFileSync(file('tt.cp').fsPath, 'var s = 0\nfor i in 0..3:\n    s += i\nprint(s)\n');
            await open('tt.cp');
            let stops = 0;
            const tracker = vscode.debug.registerDebugAdapterTrackerFactory('cp', {
                createDebugAdapterTracker: () => ({
                    onDidSendMessage: (m: { type?: string; event?: string }) => {
                        if (m.type === 'event' && m.event === 'stopped') stops++;
                    },
                }),
            });
            try {
                const started = await vscode.debug.startDebugging(vscode.workspace.workspaceFolders![0], {
                    type: 'cp',
                    request: 'launch',
                    name: 'time travel',
                    program: file('tt.cp').fsPath,
                    stopOnEntry: true,
                });
                assert.ok(started, 'the cp debug session did not start');
                const session = await until('the debug session', () => vscode.debug.activeDebugSession);
                const after = async (command: string) => {
                    // The entry stop can come before startDebugging resolves.
                    const before = command === 'launch' ? 0 : stops;
                    if (command !== 'launch') await session.customRequest(command, { threadId: 1 });
                    await until(`a stop after ${command}`, () => (stops > before ? true : undefined), 20000);
                    const trace = await session.customRequest('stackTrace', { threadId: 1 });
                    return trace.stackFrames[0].line as number;
                };
                await after('launch');
                const first = await after('next');
                const second = await after('next');
                assert.notEqual(first, second, `two steps stayed on line ${first}`);
                assert.equal(await after('stepBack'), first);
            } finally {
                tracker.dispose();
                await vscode.debug.stopDebugging();
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
            for (const word of ['domain', 'unchecked', 'lends', 'assert', 'print', 'reachable', 'MinHeap', 'Int', 'F64', 'Interactor', 'Mod', 'SegTree', 'Rooted']) {
                assert.ok(words.includes(word), `no \`${word}\` among the completions`);
            }
            for (const stale of ['INF', 'Float', 'I32', 'U8', 'ModR', 'defer', 'assume', 'where', 'inline', 'lend', 'bsearch_first', 'LazySegTree']) {
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
            // The server may spell a Windows path `c:/...`.
            const [command, file] = memory.arguments ?? [];
            assert.deepEqual([command, vscode.Uri.file(file).fsPath], ['judges', document.uri.fsPath]);
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
            if (process.platform === 'win32') {
                return; // The stub is a `#!` script, which Windows can't start.
            }
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
    // A cold CI runner can take seconds to check before `cpl` runs anything.
    await vscode.workspace.getConfiguration('cp').update('run.timeoutMs', 60000, vscode.ConfigurationTarget.Workspace);
    const failed: string[] = [];
    for (const [name, test] of tests) {
        try {
            await test();
            console.log(`ok - ${name}`);
        } catch (error) {
            failed.push(name);
            console.error(`not ok - ${name}\n${error instanceof Error ? error.stack : error}`);
        }
        // Each test starts clean: a debug session left stopped would add Replay lenses to the next.
        await vscode.debug.stopDebugging();
        await until('no debug session', () => (vscode.debug.activeDebugSession ? undefined : true));
    }
    console.log(`${tests.length - failed.length} of ${tests.length} integration tests passed`);
    if (failed.length > 0) {
        throw new Error(`failed: ${failed.join('; ')}`);
    }
}
