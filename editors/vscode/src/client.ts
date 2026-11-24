// The language client: starts `cpl lsp` and hands the editor's requests to
// it. Diagnostics, hover, go-to-definition, references, rename, the outline,
// quick fixes, completion, signature help, inlay hints, the lenses and what
// running them shows all come from the server.

import * as vscode from 'vscode';
import { LanguageClient, RevealOutputChannelOn } from 'vscode-languageclient/node';

export const SELECTOR = [
    { language: 'cp', scheme: 'file' },
    { language: 'cp', scheme: 'untitled' },
];

/** What the editor says when it could not start `cpl`. */
export const MISSING_CPL =
    "Couldn't start `cpl`. Install it from https://cp-lang.org/install (it goes on `PATH`), or point `cp.server.path` at the binary.";

/** `cp.server.path` as written: `cpl` from `PATH`, where the installer puts it. */
export function serverBinary(): string {
    return vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
}

export class CpClient implements vscode.Disposable {
    private client: LanguageClient | undefined;
    private readonly status = vscode.languages.createLanguageStatusItem('cp.server', SELECTOR);

    /** `page` opens the page a trace answers with (`cpl/page`). */
    constructor(
        private readonly output: vscode.OutputChannel,
        private readonly page: (params: { html: string }) => void,
    ) {
        this.status.name = 'CP language server';
        this.status.command = { command: 'cp.restartServer', title: 'Restart' };
    }

    async start(): Promise<void> {
        const config = vscode.workspace.getConfiguration('cp');
        const binary = serverBinary();
        const args = config.get<string[]>('server.args', ['lsp']);
        this.output.appendLine(`Using ${binary}`);
        this.status.text = 'cpl: starting';
        this.status.detail = binary;
        // The id is also the settings prefix of `cp.trace.server`.
        const client = new LanguageClient('cp', 'CP Lang', { command: binary, args }, {
            documentSelector: SELECTOR,
            // The server runs nothing in Restricted Mode, shows Replay lenses
            // during a `cp` debug session, and reads the `cp` settings.
            initializationOptions: {
                codeLens: config.get('codeLens.enabled', true),
                replayLenses: 'debugging',
                trusted: vscode.workspace.isTrusted,
            },
            synchronize: { configurationSection: 'cp' },
            outputChannel: this.output,
            // A run's panel comes as an Info log message; the server's own log is below that.
            revealOutputChannelOn: RevealOutputChannelOn.Info,
            middleware: {
                // The server runs `cpl.exec` on the files as saved, so save them first.
                executeCommand: async (command, args, next) => {
                    await vscode.workspace.saveAll(false);
                    return next(command, args);
                },
            },
        });
        this.client = client;
        client.onNotification('cpl/page', this.page);
        await client.start().then(
            () => {
                const info = client.initializeResult?.serverInfo;
                this.status.text = `${info?.name} ${info?.version}`;
            },
            error => {
                this.status.text = 'cpl: failed to start';
                this.status.severity = vscode.LanguageStatusSeverity.Error;
                this.output.appendLine(`could not start \`${binary} ${args.join(' ')}\`: ${error}`);
                void vscode.window.showErrorMessage(MISSING_CPL);
            },
        );
    }

    /** Sends `method` to the running server as the editor would, for tests and other extensions. */
    async request(method: string, params: unknown): Promise<unknown> {
        return this.client?.sendRequest(method, params);
    }

    /** Tells the server the debug session that now runs, if any: the Replay lenses follow it. */
    debugging(type: string | null): void {
        void this.client?.sendNotification('cpl/debugging', { type });
    }

    async stop(): Promise<void> {
        const client = this.client;
        this.client = undefined;
        // A server that already died has nothing left to stop.
        await client?.dispose().then(undefined, error => this.output.appendLine(`while stopping cpl: ${error}`));
    }

    async restart(): Promise<void> {
        await this.stop();
        await this.start();
    }

    dispose(): void {
        void this.stop();
        this.status.dispose();
    }
}
