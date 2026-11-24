// The commands behind the lenses: run a file on its samples, stress it
// against its brute / checker / interactor, or trace it. Each command runs
// `cpl`, writes what happened to the output panel and updates the inline
// overlays.

import { execFile } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { promisify } from 'node:util';
import * as vscode from 'vscode';
import { missingCpl, serverBinary } from './client';
import { Overlays } from './overlays';
import { viewOf } from './results';

interface CplResult {
    /** The exit status, or null when the process was killed or never started. */
    code: number | null;
    stdout: string;
    stderr: string;
    error?: NodeJS.ErrnoException & { killed?: boolean };
}

/** Runs `cpl` to completion and collects what it printed. Never rejects. */
function runCpl(binary: string, args: string[], options: { cwd: string; stdin?: string; timeoutMs?: number; signal?: AbortSignal }): Promise<CplResult> {
    const { cwd, timeoutMs: timeout, signal } = options;
    const running = promisify(execFile)(binary, args, { cwd, timeout, signal, encoding: 'utf8', maxBuffer: 1 << 30 });
    running.child.stdin?.on('error', () => undefined).end(options.stdin ?? '');
    return running.then(
        ({ stdout, stderr }) => ({ code: 0, stdout, stderr }),
        error => ({ code: typeof error.code === 'number' ? error.code : null, stdout: error.stdout ?? '', stderr: error.stderr ?? '', error }),
    );
}

export class Runner {
    /** Only one background live-value run at a time per document. */
    private readonly liveRuns = new Map<string, number>();
    private liveRunId = 0;
    /** The trace panel, reused by the next trace. */
    private tracePanel: vscode.WebviewPanel | undefined;

    constructor(
        private readonly output: vscode.OutputChannel,
        private readonly overlays: Overlays,
    ) {}

    /** Runs one sample by name, or all of them; cpl judges them and lays out the view. */
    async runSample(uri?: vscode.Uri, name?: string): Promise<void> {
        const live = vscode.workspace.getConfiguration('cp').get<string>('liveValues.input', '');
        const pick = name ? ['--case', name] : live ? ['--values-of', live] : [];
        await this.report(uri, 'run', ['--samples', '--format', 'json', ...pick]);
    }

    async runAllSamples(uri?: vscode.Uri): Promise<void> {
        await this.runSample(uri);
    }

    /** Runs a server lens's `cpl` arguments and shows what cpl lays out. */
    async exec([command, file, ...rest]: string[]): Promise<void> {
        const uri = file ? vscode.Uri.file(file) : undefined;
        if (command === 'run' && rest.includes('--trace')) {
            return this.trace(uri, rest);
        } else if (command === 'stress') {
            return this.stress(uri);
        }
        await this.report(uri, command, [...rest, '--format', 'json']);
    }

    async stress(uri?: vscode.Uri): Promise<void> {
        const extra = vscode.workspace.getConfiguration('cp').get<string[]>('stress.args', []);
        await this.report(uri, 'stress', ['--format', 'json', ...extra]);
    }

    /**
     * `cpl test --add -`: the input (the selection, else the clipboard) becomes the
     * next case in `<file>.tests/`, with the expected output when one is given, and runs.
     */
    async addTest(uri?: vscode.Uri, input?: string, expected?: string): Promise<boolean> {
        const document = await this.document(uri);
        if (!document) {
            return false;
        }
        if (input === undefined) {
            const editor = vscode.window.activeTextEditor;
            const selected = editor && !editor.selection.isEmpty ? editor.document.getText(editor.selection) : '';
            input = selected || (await vscode.env.clipboard.readText());
            if (input.trim() === '') {
                void vscode.window.showInformationMessage('Add a Test: select the input, or copy it, first.');
                return false;
            }
            expected = await vscode.window.showInputBox({
                title: 'Add a Test: the expected output',
                prompt: 'Leave empty to only record the input; `\\n` starts a new line.',
            });
            if (expected === undefined) {
                return false;
            }
            expected = expected.replace(/\\n/g, '\n');
        }
        if (!input.endsWith('\n')) {
            input += '\n';
        }
        const args = ['test', document.uri.fsPath, '--add', '-'];
        let expectFile: string | undefined;
        if (expected) {
            expectFile = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'cp-expect-')), 'out');
            fs.writeFileSync(expectFile, expected.endsWith('\n') ? expected : `${expected}\n`);
            args.push('--expect', expectFile);
        }
        this.output.show(true);
        const result = await this.cpl(document, args, 'cpl test --add', input, { allowRuntimeError: true });
        if (expectFile) {
            fs.rmSync(path.dirname(expectFile), { recursive: true, force: true });
        }
        if (!result) {
            return false;
        }
        this.printRaw(result);
        const added = /added (\S+)/.exec(result.stderr + result.stdout)?.[1];
        if (added) {
            void vscode.window.showInformationMessage(`Added ${added}${result.code === 0 ? '' : '; it fails, see the output'}`);
        }
        return added !== undefined;
    }

    /** `cpl new --listen` in a terminal: the next problem Competitive Companion sends lands in `folder`. */
    listen(folder: string): void {
        const binary = serverBinary();
        if (!binary) {
            void vscode.window.showErrorMessage(
                missingCpl(vscode.workspace.isTrusted),
            );
            return;
        }
        const port = vscode.workspace.getConfiguration('cp').get<number>('companion.port', 10043);
        vscode.window
            .createTerminal({ name: 'Competitive Companion', shellPath: binary, shellArgs: ['new', '--listen', '--port', String(port)], cwd: folder })
            .show();
        void vscode.window.showInformationMessage('Waiting for Competitive Companion: open the problem in the browser and click the extension.');
    }

    /** `cpl submit`: build what the judge gets, check it on the samples, and copy it when it is ready. */
    async submit(uri?: vscode.Uri): Promise<void> {
        const copy = vscode.workspace.getConfiguration('cp').get<boolean>('submit.copy', true);
        await this.report(uri, 'submit', ['--json', ...(copy ? ['--copy'] : [])]);
    }

    /** Runs `cpl <command>` on the file and shows the report's view as is. */
    private async report(uri: vscode.Uri | undefined, command: string, args: string[]): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        this.output.show(true);
        const title = `CP: cpl ${command} ${path.basename(document.uri.fsPath)}...`;
        const location = vscode.ProgressLocation.Window;
        const result = await vscode.window.withProgress({ location, title, cancellable: true }, (_progress, token) => {
            const controller = new AbortController();
            token.onCancellationRequested(() => controller.abort());
            const argv = [command, document.uri.fsPath, ...args];
            return this.cpl(document, argv, `cpl ${command}`, undefined, { signal: controller.signal });
        });
        const view = result && viewOf(result.stdout);
        if (!view) {
            if (result) {
                this.printRaw(result);
            }
            return;
        }
        this.output.append(view.panel);
        this.overlays.show(document.uri, view);
        if (view.status) {
            const show = view.status.ok ? vscode.window.showInformationMessage : vscode.window.showWarningMessage;
            void show(view.status.label);
        }
    }

    /** `cpl run --trace` with the lens's input and fn, drawn in the trace panel. */
    async trace(uri?: vscode.Uri, rest: string[] = ['--trace']): Promise<void> {
        const document = await this.document(uri);
        const args = document && ['run', document.uri.fsPath, ...rest, '--format', 'html'];
        const result = document && args && (await this.cpl(document, args, 'cpl run --trace'));
        if (document && result && /^\s*<(!doctype|html)/i.test(result.stdout)) {
            const title = `Trace: ${path.basename(document.uri.fsPath)}`;
            if (!this.tracePanel) {
                const column = { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true };
                this.tracePanel = vscode.window.createWebviewPanel('cp.trace', title, column, { enableScripts: true });
                this.tracePanel.onDidDispose(() => (this.tracePanel = undefined));
            }
            this.tracePanel.title = title;
            this.tracePanel.reveal(vscode.ViewColumn.Beside, true);
            this.tracePanel.webview.html = result.stdout;
        } else if (result) {
            this.output.show(true);
            this.printRaw(result);
        }
    }

    /** Recomputes the inline values of a saved file, quietly: nothing is shown on compile failure. */
    async refreshLiveValues(document: vscode.TextDocument): Promise<void> {
        const config = vscode.workspace.getConfiguration('cp');
        const binary = serverBinary();
        if (!binary || document.uri.scheme !== 'file') {
            return;
        }
        const key = document.uri.toString();
        const id = ++this.liveRunId;
        this.liveRuns.set(key, id);
        const version = document.version;
        const live = config.get<string>('liveValues.input', '');
        const args = ['run', document.uri.fsPath, '--samples', '--format', 'json', ...(live ? ['--values-of', live] : [])];
        const result = await runCpl(binary, args, {
            cwd: path.dirname(document.uri.fsPath),
            timeoutMs: config.get<number>('run.timeoutMs', 10000),
        });
        // A newer save or an edit since the run started makes this result stale.
        const view = this.liveRuns.get(key) === id && document.version === version ? viewOf(result.stdout) : undefined;
        if (view) {
            this.overlays.show(document.uri, view);
        }
    }

    /**
     * Runs `cpl`. Returns undefined, after telling the user why, when the
     * command could not do its work at all: no binary, the source has errors,
     * cancelled, or this part of the toolchain is not built yet.
     */
    private async cpl(
        document: vscode.TextDocument,
        args: string[],
        what: string,
        stdin?: string,
        options?: { allowRuntimeError?: boolean; signal?: AbortSignal },
    ): Promise<CplResult | undefined> {
        const binary = serverBinary();
        if (!binary) {
            void vscode.window.showErrorMessage(
                missingCpl(vscode.workspace.isTrusted),
            );
            return undefined;
        }
        if (document.isDirty) {
            await document.save();
        }
        const result = await runCpl(binary, args, {
            cwd: path.dirname(document.uri.fsPath),
            stdin,
            timeoutMs: vscode.workspace.getConfiguration('cp').get<number>('run.timeoutMs', 10000),
            signal: options?.signal,
        });
        if (result.error?.name === 'AbortError') {
            this.output.appendLine(`${what}: cancelled`);
            return undefined;
        }
        if (typeof result.error?.code === 'string') {
            this.output.appendLine(`${what}: could not start ${binary}: ${result.error.message}`);
            return undefined;
        }
        if (result.error?.killed) {
            this.output.appendLine(`${what}: stopped by \`cp.run.timeoutMs\``);
            return undefined;
        }
        if (result.code === 3 && /not built yet/i.test(result.stderr)) {
            // The file checked clean; this command's back end is not in this `cpl` yet.
            this.output.appendLine(result.stderr.trim());
            return undefined;
        }
        // `cpl run` exits with 3 on runtime traps (failed assert, OOB, overflow, interactor).
        if (options?.allowRuntimeError && result.code === 3) {
            return result;
        }
        // `cpl` prints diagnostics on stderr and nothing on stdout when the source has errors.
        if (result.code !== 0 && result.stdout.trim() === '') {
            this.output.appendLine(`${what}:`);
            this.output.appendLine(result.stderr.trimEnd().replace(/^/gm, '  '));
            return undefined;
        }
        return result;
    }

    private printRaw(result: CplResult): void {
        for (const text of [result.stdout, result.stderr]) {
            if (text.trim() !== '') {
                this.output.appendLine(text.trimEnd());
            }
        }
    }

    private async document(uri?: vscode.Uri): Promise<vscode.TextDocument | undefined> {
        if (uri) {
            return vscode.workspace.openTextDocument(uri);
        }
        const active = vscode.window.activeTextEditor?.document;
        return active?.languageId === 'cp' ? active : undefined;
    }

    /** The sample to use: null to run without input, undefined when the user backed out. */
}
