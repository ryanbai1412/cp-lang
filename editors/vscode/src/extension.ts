import * as path from 'node:path';
import * as vscode from 'vscode';
import { CpClient, missingCpl, serverBinary } from './client';

/** What the extension gives other extensions and its tests. */
export interface CpApi {
    request(method: string, params: unknown): Promise<unknown>;
}

// Every command forwards to the server's `cpl.exec`, which runs `cpl` and shows
// the report through the protocol: the panel in the output, the status as a
// message, the marks as inlay hints and the problem as a diagnostic.
export async function activate(context: vscode.ExtensionContext): Promise<CpApi> {
    const output = vscode.window.createOutputChannel('CP Lang');
    const config = () => vscode.workspace.getConfiguration('cp');
    let tracePanel: vscode.WebviewPanel | undefined;
    let traceEditor: vscode.TextEditor | undefined;
    const traceLine = vscode.window.createTextEditorDecorationType({ isWholeLine: true, backgroundColor: new vscode.ThemeColor('editor.rangeHighlightBackground') });
    /** Opens the page a trace answers with beside the editor, in one panel. */
    const page = (html?: string | null) => {
        if (typeof html !== 'string') {
            return;
        }
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
        tracePanel.title = `Trace: ${path.basename(String(traceEditor?.document.fileName))}`;
        tracePanel.reveal(vscode.ViewColumn.Beside, true);
        tracePanel.webview.html = html;
    };
    const client = new CpClient(output, page);
    const exec = (...args: unknown[]) => vscode.commands.executeCommand<{ stderr?: string } | undefined>('cpl.exec', ...args);
    const active = () => vscode.window.activeTextEditor?.document.uri;
    const command = (id: string, run: (...args: never[]) => unknown) =>
        vscode.commands.registerCommand(id, run as (...args: unknown[]) => unknown);

    context.subscriptions.push(
        output,
        client,
        vscode.debug.onDidStartDebugSession(() => client.refreshLenses()),
        vscode.debug.onDidTerminateDebugSession(() => client.refreshLenses()),
        command('cp.runSample', (uri = active(), name?: string) => exec('run', uri?.fsPath, '--samples', { case: name })),
        command('cp.runAllSamples', (uri = active()) => exec('run', uri?.fsPath, '--samples')),
        command('cp.stress', (uri = active()) => exec('stress', uri?.fsPath)),
        command('cp.submit', (uri = active()) => exec('submit', uri?.fsPath)),
        command('cp.trace', (uri = active()) => exec('run', uri?.fsPath, '--trace')),
        // The input (the selection, else the clipboard) becomes the next case, with the expected output when given.
        command('cp.addTest', async (uri = active(), input?: string, expected = '') => {
            if (input === undefined) {
                const editor = vscode.window.activeTextEditor;
                input = (editor && !editor.selection.isEmpty ? editor.document.getText(editor.selection) : '') || (await vscode.env.clipboard.readText());
                const prompt = 'Leave empty to only record the input; `\\n` starts a new line.';
                const typed = await vscode.window.showInputBox({ title: 'Add a Test: the expected output', prompt });
                if (typed === undefined) {
                    return false;
                }
                expected = typed;
            }
            const result = await exec('test', uri?.fsPath, '--add', '-', { stdin: input, expect: expected.replace(/\\n/g, '\n') });
            return /added \S+/.test(String(result?.stderr));
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
        command('cp.replayCall', async (target?: { frameId?: number }, call?: { fn?: string }) => {
            // From the call stack VS Code passes the frame; from a lens, the fn. The adapter picks the call and lays out its view.
            const session = vscode.debug.activeDebugSession;
            const view = await Promise.resolve(session?.customRequest('replayCall', { frameId: target?.frameId, fn: call?.fn })).then(
                body => body?.view ?? { panel: 'Replay needs a CP Lang debug session stopped at the call.\n' },
                (error: Error) => ({ panel: `Replay: ${error.message}\n` }),
            );
            return vscode.commands.executeCommand('cpl.show', vscode.Uri.file(String(session?.configuration.program)).toString(), view);
        }),
        command('cp.restartServer', () => client.restart()),
        command('cp.showOutput', () => output.show()),
        command('cp.clearOverlays', () => vscode.commands.executeCommand('cpl.clear', active()?.toString())),
        // The server runs the samples on each save while this is on.
        command('cp.toggleLiveValues', () =>
            config().update('liveValues.enabled', !config().get<boolean>('liveValues.enabled', false), vscode.ConfigurationTarget.Global),
        ),
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
