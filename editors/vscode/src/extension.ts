import { execFile } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { promisify } from 'node:util';
import * as vscode from 'vscode';
import { CpClient, missingCpl, serverBinary } from './client';
import { Overlays, viewOf } from './overlays';

/** What the extension gives other extensions and its tests. */
export interface CpApi {
    request(method: string, params: unknown): Promise<unknown>;
}

export async function activate(context: vscode.ExtensionContext): Promise<CpApi> {
    const output = vscode.window.createOutputChannel('CP Lang');
    const client = new CpClient(output);
    const overlays = new Overlays();
    const config = () => vscode.workspace.getConfiguration('cp');
    let tracePanel: vscode.WebviewPanel | undefined;
    let traceEditor: vscode.TextEditor | undefined;
    const traceLine = vscode.window.createTextEditorDecorationType({ isWholeLine: true, backgroundColor: new vscode.ThemeColor('editor.rangeHighlightBackground') });
    const liveRuns = new Map<string, number>();
    /** `cpl <args>` on the file `args[1]`, saved first: what it printed, or undefined without a `cpl`. */
    const cpl = async (args: string[], stdin?: string, signal?: AbortSignal) => {
        const binary = serverBinary();
        if (!binary) {
            void vscode.window.showErrorMessage(missingCpl(vscode.workspace.isTrusted));
            return undefined;
        }
        await vscode.workspace.textDocuments.find(d => d.isDirty && d.uri.fsPath === args[1])?.save();
        const timeout = config().get<number>('run.timeoutMs', 10000);
        const running = promisify(execFile)(binary, args, { cwd: path.dirname(args[1]), timeout, signal, encoding: 'utf8', maxBuffer: 1 << 30 });
        running.child.stdin?.on('error', () => undefined).end(stdin ?? '');
        return running.then(
            ({ stdout, stderr }) => ({ stdout, stderr }),
            error => ({ stdout: String(error.stdout ?? ''), stderr: String(error.stderr || error.message) }),
        );
    };
    /** Runs `cpl <args>` and shows what it returns as is: its `view`, its trace page, or its text. */
    const run = async (args: string[], stdin?: string) => {
        output.show(true);
        const title = `CP: cpl ${args[0]} ${path.basename(args[1])}...`;
        const location = vscode.ProgressLocation.Window;
        const result = await vscode.window.withProgress({ location, title, cancellable: true }, (_progress, token) => {
            const controller = new AbortController();
            token.onCancellationRequested(() => controller.abort());
            return cpl(args, stdin, controller.signal);
        });
        const view = result && viewOf(result.stdout);
        if (view) {
            output.append(view.panel);
            overlays.show(vscode.Uri.file(args[1]), view);
            const show = view.status?.ok ? vscode.window.showInformationMessage : vscode.window.showWarningMessage;
            void (view.status && show(view.status.label));
        } else if (result && /^\s*<(!doctype|html)/i.test(result.stdout)) {
            if (!tracePanel) {
                const column = { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true };
                tracePanel = vscode.window.createWebviewPanel('cp.trace', 'Trace', column, { enableScripts: true });
                // The page posts each step's and each clicked line, 0-based and checked; the editor shows it as is.
                tracePanel.webview.onDidReceiveMessage(({ line }: { line: number }) => {
                    traceEditor?.setDecorations(traceLine, [new vscode.Range(line, 0, line, 0)]);
                    traceEditor?.revealRange(new vscode.Range(line, 0, line, 0), vscode.TextEditorRevealType.InCenterIfOutsideViewport);
                });
                tracePanel.onDidDispose(() => {
                    tracePanel = undefined;
                    traceEditor?.setDecorations(traceLine, []);
                });
            }
            traceEditor = vscode.window.activeTextEditor;
            tracePanel.title = `Trace: ${path.basename(args[1])}`;
            tracePanel.reveal(vscode.ViewColumn.Beside, true);
            tracePanel.webview.html = result.stdout;
        } else if (result) {
            output.appendLine(`${result.stdout}${result.stderr}`.trimEnd());
        }
        return result;
    };
    const fileOf = (uri?: vscode.Uri) => {
        const active = vscode.window.activeTextEditor?.document;
        return uri?.fsPath ?? (active?.languageId === 'cp' ? active.uri.fsPath : undefined);
    };
    const on = (uri: vscode.Uri | undefined, args: (file: string) => string[]) => {
        const file = fileOf(uri);
        return file ? run(args(file)) : undefined;
    };
    const samples = (file: string, name?: string) => {
        const live = config().get<string>('liveValues.input', '');
        return ['run', file, '--samples', '--format', 'json', ...(name ? ['--case', name] : live ? ['--values-of', live] : [])];
    };
    const stressArgs = () => config().get<string[]>('stress.args', []);
    /** Redraws the inline values of a saved file, quietly; an edit or a newer run makes a result stale. */
    const refreshLiveValues = async (document: vscode.TextDocument) => {
        if (document.uri.scheme !== 'file' || !serverBinary()) {
            return;
        }
        const key = document.uri.toString();
        const [id, version] = [(liveRuns.get(key) ?? 0) + 1, document.version];
        liveRuns.set(key, id);
        const result = await cpl(samples(document.uri.fsPath));
        const view = result && liveRuns.get(key) === id && document.version === version ? viewOf(result.stdout) : undefined;
        if (view) {
            overlays.show(document.uri, view);
        }
    };
    const command = (id: string, run: (...args: never[]) => unknown) =>
        vscode.commands.registerCommand(id, run as (...args: unknown[]) => unknown);

    context.subscriptions.push(
        output,
        client,
        overlays,
        vscode.debug.onDidStartDebugSession(() => client.refreshLenses()),
        vscode.debug.onDidTerminateDebugSession(() => client.refreshLenses()),
        command('cp.runSample', (uri?: vscode.Uri, name?: string) => on(uri, file => samples(file, name))),
        command('cp.runAllSamples', (uri?: vscode.Uri) => on(uri, file => samples(file))),
        command('cp.stress', (uri?: vscode.Uri) => on(uri, file => ['stress', file, '--format', 'json', ...stressArgs()])),
        command('cp.submit', (uri?: vscode.Uri) =>
            on(uri, file => ['submit', file, '--json', ...(config().get<boolean>('submit.copy', true) ? ['--copy'] : [])]),
        ),
        // The input (the selection, else the clipboard) becomes the next case, with the expected output when given.
        command('cp.addTest', async (uri?: vscode.Uri, input?: string, expected?: string) => {
            const file = fileOf(uri);
            const editor = vscode.window.activeTextEditor;
            if (input === undefined) {
                input = (editor && !editor.selection.isEmpty ? editor.document.getText(editor.selection) : '') || (await vscode.env.clipboard.readText());
                const prompt = 'Leave empty to only record the input; `\\n` starts a new line.';
                const typed = input.trim() && (await vscode.window.showInputBox({ title: 'Add a Test: the expected output', prompt }));
                if (typed === undefined) {
                    return false;
                }
                expected = typed.replace(/\\n/g, '\n');
            }
            if (!file || input.trim() === '') {
                void vscode.window.showInformationMessage('Add a Test: select the input, or copy it, first.');
                return false;
            }
            const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'cp-expect-'));
            fs.writeFileSync(path.join(dir, 'out'), expected ?? '');
            const result = await run(['test', file, '--add', '-', ...(expected ? ['--expect', path.join(dir, 'out')] : [])], input);
            fs.rmSync(dir, { recursive: true, force: true });
            return /added \S+/.test(result?.stderr ?? '');
        }),
        command('cp.importProblem', (folder?: string) => {
            const active = vscode.window.activeTextEditor?.document.uri;
            const workspace = active ? vscode.workspace.getWorkspaceFolder(active) : vscode.workspace.workspaceFolders?.[0];
            const target = folder ?? (active?.scheme === 'file' ? path.dirname(active.fsPath) : workspace?.uri.fsPath);
            if (!target) {
                void vscode.window.showInformationMessage('Import a Problem: open a folder first; the problem lands there.');
                return;
            }
            const binary = serverBinary();
            if (!binary) {
                void vscode.window.showErrorMessage(missingCpl(vscode.workspace.isTrusted));
                return;
            }
            const shellArgs = ['new', '--listen', '--port', String(config().get<number>('companion.port', 10043))];
            vscode.window.createTerminal({ name: 'Competitive Companion', shellPath: binary, shellArgs, cwd: target }).show();
            void vscode.window.showInformationMessage('Waiting for Competitive Companion: open the problem in the browser and click the extension.');
        }),
        command('cp.trace', (uri?: vscode.Uri) => on(uri, file => ['run', file, '--trace', '--format', 'html'])),
        command('cp.debug', async (uri?: vscode.Uri) => {
            const target = uri ?? vscode.window.activeTextEditor?.document.uri;
            if (!target) {
                return;
            }
            const folder = vscode.workspace.getWorkspaceFolder(target);
            await vscode.debug.startDebugging(folder, {
                type: 'cp',
                request: 'launch',
                name: 'Debug CP Lang File',
                program: target.fsPath,
                stopOnEntry: true,
            });
        }),
        vscode.debug.registerDebugAdapterDescriptorFactory('cp', {
            createDebugAdapterDescriptor(): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
                const binary = serverBinary();
                if (!binary) {
                    void vscode.window.showErrorMessage(
                        missingCpl(vscode.workspace.isTrusted),
                    );
                    return undefined;
                }
                return new vscode.DebugAdapterExecutable(binary, ['dap']);
            },
        }),
        command('cp.replayCall', async (target?: unknown, call?: { fn?: string }) => {
            const session = vscode.debug.activeDebugSession;
            if (session?.type !== 'cp') {
                void vscode.window.showInformationMessage('Replay needs a CP Lang debug session stopped at the call.');
                return;
            }
            // From the call stack VS Code passes the frame; from a lens, the document and the fn.
            const frameId = (target as { frameId?: unknown } | undefined)?.frameId;
            const args = call?.fn ? { fn: call.fn } : typeof frameId === 'number' ? { frameId } : {};
            try {
                const body = await session.customRequest('replayCall', args);
                const report = String(body?.report ?? '');
                output.appendLine(report);
                const [first, second] = report.split('\n');
                void vscode.window.showInformationMessage(`${first}: ${second}`, 'Show report').then(choice => {
                    if (choice) {
                        output.show(true);
                    }
                });
            } catch (error) {
                void vscode.window.showErrorMessage(`Replay: ${error instanceof Error ? error.message : String(error)}`);
            }
        }),
        // A server lens's `cpl` arguments, run as they are.
        command('cpl.exec', (...args: string[]) => {
            const format = args.includes('--trace') ? 'html' : 'json';
            return run([...args, '--format', format, ...(args[0] === 'stress' ? stressArgs() : [])]);
        }),
        command('cp.restartServer', () => client.restart()),
        command('cp.showOutput', () => output.show()),
        command('cp.clearOverlays', () => {
            const uri = vscode.window.activeTextEditor?.document.uri;
            if (uri) {
                overlays.show(uri, undefined);
            }
        }),
        command('cp.toggleLiveValues', async () => {
            const config = vscode.workspace.getConfiguration('cp');
            const enabled = !config.get<boolean>('liveValues.enabled', false);
            await config.update('liveValues.enabled', enabled, vscode.ConfigurationTarget.Global);
            const document = vscode.window.activeTextEditor?.document;
            if (document?.languageId !== 'cp') {
                return;
            }
            if (enabled) {
                await refreshLiveValues(document);
            } else {
                overlays.show(document.uri, undefined);
            }
        }),
        vscode.workspace.onDidSaveTextDocument(document => {
            const enabled = vscode.workspace.getConfiguration('cp').get<boolean>('liveValues.enabled', false);
            if (enabled && document.languageId === 'cp') {
                void refreshLiveValues(document);
            }
        }),
        vscode.workspace.onDidGrantWorkspaceTrust(() => void client.restart()),
        vscode.workspace.onDidChangeConfiguration(event => {
            if (event.affectsConfiguration('cp.server')) {
                void client.restart();
            }
        }),
    );
    await client.start();
    return { request: (method, params) => client.request(method, params) };
}

export function deactivate(): void {
    // The client is in `context.subscriptions`, which stops the server.
}
