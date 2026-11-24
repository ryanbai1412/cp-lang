import * as path from 'node:path';
import * as vscode from 'vscode';
import { CpClient, serverBinary } from './client';

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
    /** Opens the page a trace answers with beside the editor, in place of the last one. */
    const page = ({ html }: { html: string }) => {
        tracePanel?.dispose();
        traceEditor = vscode.window.activeTextEditor;
        const title = `Trace: ${path.basename(String(traceEditor?.document.fileName))}`;
        const panel = vscode.window.createWebviewPanel('cp.trace', title, { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true }, { enableScripts: true });
        // The page posts each step's and each clicked line, 0-based and checked; the editor shows it as is.
        panel.webview.onDidReceiveMessage(({ line }: { line: number }) => {
            traceEditor?.setDecorations(traceLine, [new vscode.Range(line, 0, line, 0)]);
            traceEditor?.revealRange(new vscode.Range(line, 0, line, 0), vscode.TextEditorRevealType.InCenterIfOutsideViewport);
        });
        panel.onDidDispose(() => traceEditor?.setDecorations(traceLine, []));
        panel.webview.html = html;
        tracePanel = panel;
    };
    const client = new CpClient(output, page);
    const exec = (...args: unknown[]) => vscode.commands.executeCommand<{ ok: boolean } | undefined>('cpl.exec', ...args);
    const active = () => vscode.window.activeTextEditor?.document.uri;
    const command = (id: string, run: (...args: never[]) => unknown) =>
        vscode.commands.registerCommand(id, run as (...args: unknown[]) => unknown);
    const selected = (editor = vscode.window.activeTextEditor) => editor?.document.getText(editor?.selection);

    context.subscriptions.push(
        output,
        client,
        vscode.debug.onDidStartDebugSession(session => client.debugging(session.type)),
        vscode.debug.onDidTerminateDebugSession(() => client.debugging(null)),
        command('cp.runSample', (uri = active(), name: unknown) => exec('run', uri?.fsPath, '--samples', { case: name })),
        command('cp.runAllSamples', (uri = active()) => exec('run', uri?.fsPath, '--samples')),
        command('cp.stress', (uri = active()) => exec('stress', uri?.fsPath)),
        command('cp.submit', (uri = active()) => exec('submit', uri?.fsPath)),
        command('cp.trace', (uri = active()) => exec('run', uri?.fsPath, '--trace')),
        // The input (the selection, else the clipboard) becomes the next case, with the expected output typed.
        command(
            'cp.addTest',
            async (
                uri = active(),
                stdin: unknown = Promise.all([selected(), vscode.env.clipboard.readText()]),
                typed: unknown = vscode.window.showInputBox({
                    title: 'Add a Test: the expected output',
                    prompt: 'Leave empty to only record the input; `\\n` starts a new line.',
                }),
            ) => (await exec('test', uri?.fsPath, '--add', '-', { stdin: await stdin, typed: [await typed] }))?.ok,
        ),
        command('cp.importProblem', (folder: unknown = active()?.fsPath) =>
            vscode.commands.executeCommand('cpl.import', folder, config().get('companion.port', 10043)),
        ),
        command('cp.debug', (uri = active()) =>
            vscode.debug.startDebugging(undefined, {
                type: 'cp',
                request: 'launch',
                name: 'Debug CP Lang File',
                program: uri?.fsPath,
                stopOnEntry: true,
            }),
        ),
        // VS Code runs no debugger in Restricted Mode.
        vscode.debug.registerDebugAdapterDescriptorFactory('cp', {
            createDebugAdapterDescriptor: () => new vscode.DebugAdapterExecutable(serverBinary(), ['dap']),
        }),
        // From the call stack VS Code passes the frame; from a lens, the fn. The adapter picks the call and lays out its view.
        command('cp.replayCall', async (target: { frameId: number } | undefined, call: { fn: string } | undefined) => {
            const session = vscode.debug.activeDebugSession;
            const body = await session?.customRequest('replayCall', { frameId: target?.frameId, fn: call?.fn });
            return vscode.commands.executeCommand('cpl.show', vscode.Uri.file(String(session?.configuration.program)).toString(), body?.view);
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
