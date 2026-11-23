import * as path from 'node:path';
import * as vscode from 'vscode';
import { CpClient, SELECTOR, serverBinary } from './client';
import { installHint } from './cpl';
import { LensProvider } from './lenses';
import { Overlays } from './overlays';
import { Runner } from './runner';
import { Sample } from './samples';
import { TestPanel } from './testPanel';
import { TraceView } from './traceView';

/** What the extension gives other extensions and its tests. */
export interface CpApi {
    request(method: string, params: unknown): Promise<unknown>;
    /** The Testing panel's problems and cases. */
    tests: TestPanel;
}

export async function activate(context: vscode.ExtensionContext): Promise<CpApi> {
    const output = vscode.window.createOutputChannel('CP Lang');
    const client = new CpClient(output);
    const overlays = new Overlays();
    const lenses = new LensProvider();
    const traces = new TraceView();
    const runner = new Runner(output, overlays, traces);
    const tests = new TestPanel();
    const command = (id: string, run: (...args: never[]) => unknown) =>
        vscode.commands.registerCommand(id, run as (...args: unknown[]) => unknown);

    context.subscriptions.push(
        output,
        client,
        overlays,
        traces,
        lenses,
        tests,
        vscode.languages.registerCodeLensProvider(SELECTOR, lenses),
        command('cp.runSample', (uri?: vscode.Uri, sample?: Sample) => runner.runSample(uri, sample)),
        command('cp.runAllSamples', (uri?: vscode.Uri) => runner.runAllSamples(uri)),
        command('cp.stress', (uri?: vscode.Uri, part?: string) => runner.stress(uri, part)),
        command('cp.submit', (uri?: vscode.Uri) => runner.submit(uri)),
        command('cp.addTest', (uri?: vscode.Uri, input?: string, expected?: string) => runner.addTest(uri, input, expected)),
        command('cp.importProblem', (folder?: string) => {
            const active = vscode.window.activeTextEditor?.document.uri;
            const workspace = active ? vscode.workspace.getWorkspaceFolder(active) : vscode.workspace.workspaceFolders?.[0];
            const target = folder ?? (active?.scheme === 'file' ? path.dirname(active.fsPath) : workspace?.uri.fsPath);
            if (!target) {
                void vscode.window.showInformationMessage('Import a Problem: open a folder first; the problem lands there.');
                return;
            }
            runner.listen(target);
        }),
        command('cp.trace', (uri?: vscode.Uri, sample?: Sample, part?: string) => runner.trace(uri, sample, part)),
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
                        `Couldn't find \`cpl\`. ${installHint()}`,
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
        // The server's lenses name `cpl` arguments; run them in a terminal.
        command('cpl.exec', (...args: string[]) => {
            const binary = serverBinary();
            if (binary) {
                vscode.window.createTerminal({ name: 'cpl', shellPath: binary, shellArgs: args }).show();
            }
        }),
        command('cp.restartServer', () => client.restart()),
        command('cp.showOutput', () => output.show()),
        command('cp.clearOverlays', () => {
            const uri = vscode.window.activeTextEditor?.document.uri;
            if (uri) {
                overlays.clear(uri);
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
                await runner.refreshLiveValues(document);
            } else {
                overlays.clear(document.uri);
            }
        }),
        vscode.workspace.onDidSaveTextDocument(document => {
            const enabled = vscode.workspace.getConfiguration('cp').get<boolean>('liveValues.enabled', false);
            if (enabled && document.languageId === 'cp') {
                void runner.refreshLiveValues(document);
            }
        }),
        vscode.workspace.onDidChangeConfiguration(event => {
            if (event.affectsConfiguration('cp.server')) {
                void client.restart();
            }
        }),
    );
    await client.start();
    return { request: (method, params) => client.request(method, params), tests };
}

export function deactivate(): void {
    // The client is in `context.subscriptions`, which stops the server.
}
