// The language client: starts `cpl lsp` and hands the editor's requests to
// it. Diagnostics, hover, go-to-definition, the outline and quick fixes come
// from the server's capabilities, and completion will too once the server
// advertises it; until then `completion.ts` fills in.

import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    RevealOutputChannelOn,
    ServerOptions,
    State,
} from 'vscode-languageclient/node';
import { resolveServer } from './cpl';
import { registerFallbackCompletion } from './completion';

export const SELECTOR = [
    { language: 'cp', scheme: 'file' },
    { language: 'cp', scheme: 'untitled' },
];

/** The `cpl` binary the settings point at, or undefined when it can't be found. */
export function serverBinary(): string | undefined {
    const configured = vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
    return resolveServer(configured, {
        pathEnv: process.env.PATH ?? '',
        workspaceFolders: (vscode.workspace.workspaceFolders ?? []).map(folder => folder.uri.fsPath),
    });
}

export class CpClient implements vscode.Disposable {
    private client: LanguageClient | undefined;
    private fallback: vscode.Disposable | undefined;
    private readonly status: vscode.LanguageStatusItem;
    private warnedMissing = false;

    constructor(private readonly output: vscode.OutputChannel) {
        this.status = vscode.languages.createLanguageStatusItem('cp.server', SELECTOR);
        this.status.name = 'CP language server';
        this.status.command = { command: 'cp.restartServer', title: 'Restart' };
        this.setStatus('cpl: stopped', vscode.LanguageStatusSeverity.Warning);
    }

    async start(): Promise<void> {
        const binary = serverBinary();
        if (!binary) {
            this.missingBinary();
            return;
        }
        const args = vscode.workspace.getConfiguration('cp').get<string[]>('server.args', ['lsp']);
        const serverOptions: ServerOptions = {
            run: { command: binary, args },
            debug: { command: binary, args },
        };
        const clientOptions: LanguageClientOptions = {
            documentSelector: SELECTOR,
            outputChannel: this.output,
            revealOutputChannelOn: RevealOutputChannelOn.Never,
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
                this.setStatus('cpl: stopped', vscode.LanguageStatusSeverity.Warning);
            }
        });
        try {
            await client.start();
        } catch (error) {
            this.setStatus('cpl: failed to start', vscode.LanguageStatusSeverity.Error);
            this.output.appendLine(`could not start \`${binary} ${args.join(' ')}\`: ${error}`);
            return;
        }
        const info = client.initializeResult?.serverInfo;
        this.setStatus(
            info ? `${info.name} ${info.version ?? ''}`.trim() : 'cpl',
            vscode.LanguageStatusSeverity.Information,
            binary,
        );
        if (!client.initializeResult?.capabilities.completionProvider) {
            this.fallback = registerFallbackCompletion(SELECTOR);
        }
    }

    async stop(): Promise<void> {
        this.fallback?.dispose();
        this.fallback = undefined;
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

    dispose(): void {
        void this.stop();
        this.status.dispose();
    }

    private setStatus(text: string, severity: vscode.LanguageStatusSeverity, detail?: string): void {
        this.status.busy = false;
        this.status.text = text;
        this.status.severity = severity;
        this.status.detail = detail;
    }

    private missingBinary(): void {
        const configured = vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
        this.setStatus('cpl: not found', vscode.LanguageStatusSeverity.Error, `\`${configured}\` is not on PATH`);
        // Highlighting works without the server, so say it once and stay out of the way.
        if (this.warnedMissing) {
            return;
        }
        this.warnedMissing = true;
        const choose = 'Set the Path';
        void vscode.window
            .showWarningMessage(
                `Couldn't find \`${configured}\`, so errors, hover and quick fixes are off. ` +
                    'Build it with `cargo build --release -p cpl-cli`, or point `cp.server.path` at the binary.',
                choose,
            )
            .then(choice => {
                if (choice === choose) {
                    void vscode.commands.executeCommand('workbench.action.openSettings', 'cp.server.path');
                }
            });
    }
}
