// The language client: starts `cpl lsp` and hands the editor's requests to
// it. Diagnostics, hover, go-to-definition, references, rename, the outline,
// quick fixes, completion, signature help and inlay hints all come from the
// server's capabilities.

import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
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

const INSTALL =
    process.platform === 'win32'
        ? 'Install it with `irm https://cp-lang.org/install.ps1 | iex` in PowerShell, or point `cp.server.path` at the binary.'
        : 'Install it with `curl -fsSL https://cp-lang.org/install.sh | sh`, or point `cp.server.path` at the binary.';

/** Why CP Lang doesn't run `cpl` in a folder VS Code hasn't been told to trust. */
const RESTRICTED =
    'This folder is in Restricted Mode, so CP Lang highlights .cp files but does not run `cpl`: no diagnostics, completion or runs. Trust the folder to turn them on.';

/** What to tell the contestant when there is no `cpl` to run. */
export function missingCpl(trusted: boolean): string {
    return trusted ? `Couldn't find \`cpl\`. ${INSTALL}` : RESTRICTED;
}

/**
 * The `cpl` binary the settings point at, or undefined when it can't be found.
 * A path is taken as written, relative to each workspace folder. A bare name is
 * looked up on `PATH`, where the installer and cargo put it (an editor started
 * from the dock doesn't see the shell's `PATH`), then in each folder's newer
 * `target/` build, so the extension works inside the cp-lang repository.
 */
export function serverBinary(): string | undefined {
    const configured = vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
    if (!vscode.workspace.isTrusted || configured.trim() === '') {
        return undefined;
    }
    const folders = (vscode.workspace.workspaceFolders ?? []).map(folder => folder.uri.fsPath);
    const names = process.platform === 'win32' && !/\.exe$/i.test(configured) ? [configured, `${configured}.exe`] : [configured];
    const at = (dir: string) => names.map(name => path.resolve(dir, name)).find(file => fs.statSync(file, { throwIfNoEntry: false })?.isFile());
    if (path.isAbsolute(configured) || /[\\/]/.test(configured)) {
        return path.isAbsolute(configured) ? at('/') : folders.map(at).find(Boolean);
    }
    const home = os.homedir();
    const built = (folder: string) =>
        ['release', 'debug']
            .map(profile => path.join(folder, 'target', profile))
            .sort((a, b) => (fs.statSync(path.join(b, names[0]), { throwIfNoEntry: false })?.mtimeMs ?? 0) - (fs.statSync(path.join(a, names[0]), { throwIfNoEntry: false })?.mtimeMs ?? 0));
    const dirs = [
        ...(process.env.PATH ?? '').split(path.delimiter).filter(Boolean),
        ...(process.env.CPL_HOME ? [path.join(process.env.CPL_HOME, 'bin')] : []),
        path.join(home, '.cpl', 'bin'),
        path.join(home, '.cargo', 'bin'),
        ...folders.flatMap(built),
    ];
    return dirs.map(at).find(Boolean);
}

export class CpClient implements vscode.Disposable {
    private client: LanguageClient | undefined;
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
            outputChannel: this.output,
            revealOutputChannelOn: RevealOutputChannelOn.Never,
            middleware: {
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

    private missingBinary(): void {
        const configured = vscode.workspace.getConfiguration('cp').get<string>('server.path', 'cpl');
        this.status.command = {
            command: 'workbench.action.openSettings',
            title: 'Set Path',
            arguments: ['cp.server.path'],
        };
        this.setStatus('cpl: not found', vscode.LanguageStatusSeverity.Error, `\`${configured}\` is not on PATH`);
        // Highlighting and snippets still work without the server.
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
                    INSTALL,
                choose,
            )
            .then(choice => {
                if (choice === choose) {
                    void vscode.commands.executeCommand('workbench.action.openSettings', 'cp.server.path');
                }
            });
    }
}
