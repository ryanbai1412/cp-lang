import * as os from 'node:os';
// The language client: starts `cpl lsp` and hands the editor's requests to
// it. Diagnostics, hover, go-to-definition, references, rename, the outline,
// quick fixes, completion, signature help and inlay hints all come from the
// server's capabilities. `completion.ts` and `inlayHints.ts` fill in only
// while no server is running or one advertises neither provider.

import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    RevealOutputChannelOn,
    ServerOptions,
    State,
} from 'vscode-languageclient/node';
import { inAttribute, registerDecoratorCompletion, registerFallbackCompletion } from './completion';
import { installHint, RESTRICTED, resolveServer } from './cpl';
import { registerFallbackInlayHints } from './inlayHints';

export const SELECTOR = [
    { language: 'cp', scheme: 'file' },
    { language: 'cp', scheme: 'untitled' },
];

/** The `cpl` binary the settings point at, or undefined when it can't be found. */
export function serverBinary(): string | undefined {
    if (!vscode.workspace.isTrusted) {
        return undefined;
    }
    const configured = vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
    return resolveServer(configured, {
        pathEnv: process.env.PATH ?? '',
        workspaceFolders: (vscode.workspace.workspaceFolders ?? []).map(folder => folder.uri.fsPath),
        home: os.homedir(),
        cplHome: process.env.CPL_HOME,
    });
}

export class CpClient implements vscode.Disposable {
    private client: LanguageClient | undefined;
    private readonly decorators = registerDecoratorCompletion(SELECTOR);
    private fallbackCompletion: vscode.Disposable | undefined;
    private fallbackInlayHints: vscode.Disposable | undefined;
    private readonly status: vscode.LanguageStatusItem;
    private warnedMissing = false;
    /** The `cpl` the server runs, shown in the status and the output. */
    private binary: string | undefined;

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
            this.ensureFallbacks({ completion: true, inlayHints: true });
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
            outputChannel: this.output,
            revealOutputChannelOn: RevealOutputChannelOn.Never,
            middleware: {
                // After `@` the decorators provider answers alone.
                provideCompletionItem: (document, position, context, token, next) =>
                    inAttribute(document.lineAt(position.line).text.slice(0, position.character))
                        ? undefined
                        : next(document, position, context, token),
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
                this.ensureFallbacks({ completion: true, inlayHints: true });
            }
        });
        try {
            await client.start();
        } catch (error) {
            this.setStatus('cpl: failed to start', vscode.LanguageStatusSeverity.Error, binary);
            this.output.appendLine(`could not start \`${binary} ${args.join(' ')}\`: ${error}`);
            this.ensureFallbacks({ completion: true, inlayHints: true });
            return;
        }
        const info = client.initializeResult?.serverInfo;
        const caps = client.initializeResult?.capabilities;
        const fallbacks = [!caps?.completionProvider && 'completion', !caps?.inlayHintProvider && 'inlay hints'].filter(
            (x): x is string => Boolean(x),
        );
        const detail = fallbacks.length > 0 ? `${binary} (local fallback: ${fallbacks.join(', ')})` : binary;
        this.setStatus(
            info ? `${info.name} ${info.version ?? ''}`.trim() : 'cpl',
            vscode.LanguageStatusSeverity.Information,
            detail,
        );
        this.ensureFallbacks({
            completion: !caps?.completionProvider,
            inlayHints: !caps?.inlayHintProvider,
        });
    }

    /** Sends `method` to the running server as the editor would, for tests and other extensions. */
    async request(method: string, params: unknown): Promise<unknown> {
        if (!this.client?.isRunning()) {
            throw new Error('cpl lsp is not running');
        }
        return this.client.sendRequest(method, params);
    }

    async stop(): Promise<void> {
        this.disposeFallbacks();
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
        this.decorators.dispose();
        this.status.dispose();
    }

    private ensureFallbacks(need: { completion: boolean; inlayHints: boolean }): void {
        if (need.completion && !this.fallbackCompletion) {
            this.fallbackCompletion = registerFallbackCompletion(SELECTOR);
        } else if (!need.completion && this.fallbackCompletion) {
            this.fallbackCompletion.dispose();
            this.fallbackCompletion = undefined;
        }
        if (need.inlayHints && !this.fallbackInlayHints) {
            this.fallbackInlayHints = registerFallbackInlayHints(SELECTOR);
        } else if (!need.inlayHints && this.fallbackInlayHints) {
            this.fallbackInlayHints.dispose();
            this.fallbackInlayHints = undefined;
        }
    }

    private disposeFallbacks(): void {
        this.fallbackCompletion?.dispose();
        this.fallbackCompletion = undefined;
        this.fallbackInlayHints?.dispose();
        this.fallbackInlayHints = undefined;
    }

    private setStatus(text: string, severity: vscode.LanguageStatusSeverity, detail?: string): void {
        this.status.busy = false;
        this.status.text = text;
        this.status.severity = severity;
        this.status.detail = detail ?? this.binary;
    }

    private missingBinary(): void {
        const configured = vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
        this.status.command = {
            command: 'workbench.action.openSettings',
            title: 'Set Path',
            arguments: ['cp.server.path'],
        };
        this.setStatus('cpl: not found', vscode.LanguageStatusSeverity.Error, `\`${configured}\` is not on PATH`);
        // Highlighting, snippets and fallback completion/inlay hints still work without the server.
        if (this.warnedMissing) {
            return;
        }
        this.warnedMissing = true;
        if (!vscode.workspace.isTrusted) {
            this.setStatus('cpl: Restricted Mode', vscode.LanguageStatusSeverity.Warning, RESTRICTED);
            const trust = 'Manage Workspace Trust';
            void vscode.window.showWarningMessage(RESTRICTED, trust).then(choice => {
                if (choice === trust) {
                    void vscode.commands.executeCommand('workbench.trust.manage');
                }
            });
            return;
        }
        const choose = 'Set the Path';
        void vscode.window
            .showWarningMessage(
                `Couldn't find \`${configured}\`, so live diagnostics, hover and quick fixes are off. ` +
                    installHint(),
                choose,
            )
            .then(choice => {
                if (choice === choose) {
                    void vscode.commands.executeCommand('workbench.action.openSettings', 'cp.server.path');
                }
            });
    }
}
