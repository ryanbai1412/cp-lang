// The language client: starts `cpl lsp` and hands the editor's requests to
// it. Diagnostics, hover, go-to-definition, references, rename, the outline,
// quick fixes, completion, signature help and inlay hints all come from the
// server's capabilities.

import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    RevealOutputChannelOn,
    ServerOptions,
    State,
} from 'vscode-languageclient/node';

export const SELECTOR = [
    { language: 'cp', scheme: 'file' },
    { language: 'cp', scheme: 'untitled' },
];

/** What the editor says when it may not, or could not, start `cpl`. */
export function missingCpl(trusted: boolean): string {
    return trusted
        ? "Couldn't start `cpl`. Install it from https://cp-lang.org/install (it goes on `PATH`), or point `cp.server.path` at the binary."
        : 'This folder is in Restricted Mode, so CP Lang highlights .cp files but does not run `cpl`. Trust the folder to turn it on.';
}

/** `cp.server.path` as written, in a trusted workspace: `cpl` from `PATH`, where the installer puts it. */
export function serverBinary(): string | undefined {
    return vscode.workspace.isTrusted ? vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl') : undefined;
}

export class CpClient implements vscode.Disposable {
    private client: LanguageClient | undefined;
    private readonly status: vscode.LanguageStatusItem;
    private warnedMissing = false;
    /** The `cpl` the server runs, shown in the status and the output. */
    private binary: string | undefined;

    /** `page` opens the page a trace answers with. */
    constructor(
        private readonly output: vscode.OutputChannel,
        private readonly page: (html?: string | null) => void,
    ) {
        this.status = vscode.languages.createLanguageStatusItem('cp.server', SELECTOR);
        this.status.name = 'CP language server';
        this.status.command = { command: 'cp.restartServer', title: 'Restart' };
        this.setStatus('cpl: stopped', vscode.LanguageStatusSeverity.Warning);
    }

    async start(): Promise<void> {
        const binary = serverBinary();
        if (!binary) {
            this.setStatus('cpl: Restricted Mode', vscode.LanguageStatusSeverity.Warning, missingCpl(false));
            this.warnOnce(missingCpl(false), 'Manage Workspace Trust', 'workbench.trust.manage');
            return;
        }
        this.binary = binary;
        this.output.appendLine(`Using ${binary}`);
        this.status.command = { command: 'cp.restartServer', title: 'Restart' };
        const args = vscode.workspace.getConfiguration('cp').get<string[]>('server.args', ['lsp']);
        const serverOptions: ServerOptions = {
            run: { command: binary, args },
            debug: { command: binary, args },
        };
        const clientOptions: LanguageClientOptions = {
            documentSelector: SELECTOR,
            // The server's Run, Trace, Test, Stress and Replay lenses; `cpl.exec` runs them.
            initializationOptions: {
                codeLens: vscode.workspace.getConfiguration('cp').get('codeLens.enabled', true),
                replayLenses: true,
            },
            // The server reads the `cp` settings that complete what it runs.
            synchronize: { configurationSection: 'cp' },
            outputChannel: this.output,
            // A run's panel comes as an Info log message; the server's own log is below that.
            revealOutputChannelOn: RevealOutputChannelOn.Info,
            middleware: {
                // The server runs `cpl.exec` on the files as saved, so save them first.
                executeCommand: async (command, args, next) => {
                    await vscode.workspace.saveAll(false);
                    const result = await next(command, args);
                    this.page(result?.html);
                    return result;
                },
                // Replay lenses only while a CP Lang debug session runs.
                provideCodeLenses: async (document, token, next) =>
                    vscode.debug.activeDebugSession?.type === 'cp'
                        ? next(document, token)
                        : ((await next(document, token)) ?? []).filter(lens => lens.command?.command !== 'cp.replayCall'),
                // `cp.inlayHints.enabled` turns off the server's hints too.
                provideInlayHints: (document, range, token, next) =>
                    vscode.workspace.getConfiguration('cp').get<boolean>('inlayHints.enabled', true)
                        ? next(document, range, token)
                        : [],
            },
        };
        // The id is also the settings prefix of `cp.trace.server`.
        const client = new LanguageClient('cp', 'CP Lang', serverOptions, clientOptions);
        this.client = client;
        client.onDidChangeState(event => {
            if (this.client !== client) {
                return;
            }
            if (event.newState === State.Starting) {
                this.setStatus('cpl: starting', vscode.LanguageStatusSeverity.Information);
                this.status.busy = true;
            } else if (event.newState === State.Stopped) {
                this.setStatus(
                    'cpl: stopped',
                    vscode.LanguageStatusSeverity.Warning,
                    'Language server stopped — click Restart',
                );
            }
        });
        try {
            await client.start();
        } catch (error) {
            this.setStatus('cpl: failed to start', vscode.LanguageStatusSeverity.Error, binary);
            this.output.appendLine(`could not start \`${binary} ${args.join(' ')}\`: ${error}`);
            this.warnOnce(missingCpl(true), 'Set the Path', 'workbench.action.openSettings', 'cp.server.path');
            return;
        }
        const info = client.initializeResult?.serverInfo;
        this.setStatus(
            info ? `${info.name} ${info.version ?? ''}`.trim() : 'cpl',
            vscode.LanguageStatusSeverity.Information,
            binary,
        );
    }

    /** Sends `method` to the running server as the editor would, for tests and other extensions. */
    async request(method: string, params: unknown): Promise<unknown> {
        if (!this.client?.isRunning()) {
            throw new Error('cpl lsp is not running');
        }
        return this.client.sendRequest(method, params);
    }

    async stop(): Promise<void> {
        const client = this.client;
        this.client = undefined;
        try {
            if (client?.isRunning()) {
                await client.stop();
            }
            await client?.dispose();
        } catch (error) {
            // A server that already died has nothing left to stop.
            this.output.appendLine(`while stopping cpl: ${error}`);
        }
    }

    async restart(): Promise<void> {
        await this.stop();
        this.warnedMissing = false;
        await this.start();
    }

    /** Asks for the lenses again: debugging turns the Replay lenses on and off. */
    refreshLenses(): void {
        const feature = this.client?.getFeature('textDocument/codeLens');
        for (const editor of vscode.window.visibleTextEditors) {
            feature?.getProvider(editor.document)?.onDidChangeCodeLensEmitter.fire();
        }
    }

    dispose(): void {
        void this.stop();
        this.status.dispose();
    }

    private setStatus(text: string, severity: vscode.LanguageStatusSeverity, detail?: string): void {
        this.status.busy = false;
        this.status.text = text;
        this.status.severity = severity;
        this.status.detail = detail ?? this.binary;
    }

    /** Highlighting and snippets still work without the server, so say it once. */
    private warnOnce(message: string, action: string, command: string, ...args: string[]): void {
        if (!this.warnedMissing) {
            this.warnedMissing = true;
            void vscode.window.showWarningMessage(message, action).then(choice => choice && vscode.commands.executeCommand(command, ...args));
        }
    }
}
