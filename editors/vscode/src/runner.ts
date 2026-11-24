// The commands behind the lenses: run a file on its samples, stress it
// against its brute / checker / interactor, or trace it. Each command runs
// `cpl`, writes what happened to the output panel and updates the inline
// overlays.

import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { serverBinary } from './client';
import { CplResult, EXIT_FAILED, EXIT_NOT_BUILT, missingCpl, runCpl } from './cpl';
import { Overlays } from './overlays';
import {
    parseRunReport,
    viewOf,
} from './results';
import { Sample } from './samples';
import { TraceView } from './traceView';

export class Runner {
    /** Only one background live-value run at a time per document. */
    private readonly liveRuns = new Map<string, number>();
    private liveRunId = 0;

    constructor(
        private readonly output: vscode.OutputChannel,
        private readonly overlays: Overlays,
        private readonly traces: TraceView,
    ) {}

    /** Runs one sample, by the name `cpl run --samples --list` gives it; asks when not given. */
    async runSample(uri?: vscode.Uri, name?: string): Promise<void> {
        const document = await this.document(uri);
        const chosen = document && (name ?? (await this.pickSample(document))?.label);
        if (document && chosen !== undefined) {
            await this.runSamples(document, chosen ? ['--case', chosen] : []);
        }
    }

    /** Runs a server lens's `cpl` arguments: runs, traces and stresses draw here, the rest go to a terminal. */
    async exec(args: string[]): Promise<void> {
        const [verb, file, ...rest] = args;
        const uri = file ? vscode.Uri.file(file) : undefined;
        const input = rest[0]?.startsWith('--') ? undefined : rest[0];
        const at = rest.indexOf('--trace');
        if (verb === 'run' && at >= 0) {
            const part = rest[at + 1];
            return this.trace(uri, input ?? null, part?.startsWith('--') ? undefined : part);
        } else if (verb === 'run' && rest.includes('--samples')) {
            return this.runAllSamples(uri);
        } else if (verb === 'run') {
            const name = input && (await this.samples(await this.document(uri))).find(s => s.input === input)?.label;
            return this.runSample(uri, name ?? '');
        } else if (verb === 'stress') {
            return this.stress(uri);
        }
        const binary = serverBinary();
        if (binary) {
            vscode.window.createTerminal({ name: 'cpl', shellPath: binary, shellArgs: args }).show();
        }
    }

    async runAllSamples(uri?: vscode.Uri): Promise<void> {
        const document = await this.document(uri);
        const live = vscode.workspace.getConfiguration('cp').get<string>('liveValues.input', '');
        if (document) {
            await this.runSamples(document, live ? ['--values-of', live] : []);
        }
    }

    /** `cpl run --samples`: cpl runs and judges the samples and lays out what to show. */
    private async runSamples(document: vscode.TextDocument, extra: string[]): Promise<void> {
        this.output.show(true);
        const args = ['run', document.uri.fsPath, '--samples', '--format', 'json', ...extra];
        const result = await vscode.window.withProgress(
            { location: vscode.ProgressLocation.Window, title: 'CP: Running samples...', cancellable: true },
            async (_progress, token) => {
                const controller = new AbortController();
                const cancelSub = token.onCancellationRequested(() => controller.abort());
                try {
                    return await this.cpl(document, args, 'cpl run', undefined, { allowRuntimeError: true, signal: controller.signal });
                } finally {
                    cancelSub.dispose();
                }
            },
        );
        const view = result && viewOf(result.stdout);
        if (!view) {
            if (result) {
                this.printRaw(result);
            }
            return;
        }
        this.output.append(view.panel);
        this.overlays.show(document.uri, view);
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

    /** `input`: the sample's input file, null for none, undefined to ask. */
    async trace(uri?: vscode.Uri, input?: string | null, part?: string): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        const chosen = input !== undefined ? input : (await this.pickSample(document))?.input ?? null;
        const args = ['run', document.uri.fsPath, '--trace', ...(part ? [part] : []), '--format', 'html'];
        const stdin = chosen ? await fs.promises.readFile(chosen, 'utf8') : '';
        const result = await this.cpl(document, args, 'cpl run --trace', stdin);
        if (!result) {
            return;
        }
        if (!/^\s*<(!doctype|html)/i.test(result.stdout)) {
            this.output.show(true);
            this.printRaw(result);
            return;
        }
        this.traces.show(document, result.stdout);
    }

    /** Recomputes the inline values of a saved file, quietly: nothing is shown on compile failure. */
    async refreshLiveValues(document: vscode.TextDocument): Promise<void> {
        const config = vscode.workspace.getConfiguration('cp');
        const binary = serverBinary();
        if (!binary || document.uri.scheme !== 'file') {
            return;
        }
        const sample = this.liveSample(await this.samples(document));
        if (!sample) {
            return;
        }
        const key = document.uri.toString();
        const id = ++this.liveRunId;
        this.liveRuns.set(key, id);
        const version = document.version;
        const result = await runCpl(binary, ['run', document.uri.fsPath, '--format', 'json', '--sample', sample.label], {
            cwd: path.dirname(document.uri.fsPath),
            stdin: await fs.promises.readFile(sample.input, 'utf8'),
            timeoutMs: config.get<number>('run.timeoutMs', 10000),
        });
        // A newer save or an edit since the run started makes this result stale.
        if (this.liveRuns.get(key) !== id || document.version !== version) {
            return;
        }
        const report = parseRunReport(result.stdout);
        if (report) {
            this.overlays.show(document.uri, report.view);
        }
    }

    /** The sample whose values are shown inline: `cp.liveValues.input`, or the first. */
    private liveSample(samples: readonly Sample[]): Sample | undefined {
        const wanted = vscode.workspace.getConfiguration('cp').get<string>('liveValues.input', '');
        return wanted === '' ? samples[0] : samples.find(sample => sample.label === wanted);
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
        if (result.cancelled) {
            this.output.appendLine(`${what}: cancelled`);
            return undefined;
        }
        if (result.spawnError) {
            this.output.appendLine(`${what}: could not start ${binary}: ${result.spawnError}`);
            return undefined;
        }
        if (result.timedOut) {
            this.output.appendLine(`${what}: stopped after ${result.timeMs} ms (\`cp.run.timeoutMs\`)`);
            return undefined;
        }
        if (result.code === EXIT_NOT_BUILT && /not built yet/i.test(result.stderr)) {
            // The file checked clean; this command's back end is not in this `cpl` yet.
            this.output.appendLine(result.stderr.trim());
            return undefined;
        }
        // `cpl run` exits with EXIT_FAILED (3) on runtime traps (failed assert, OOB, overflow, interactor).
        if (options?.allowRuntimeError && result.code === EXIT_FAILED) {
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
    private async pickSample(document: vscode.TextDocument): Promise<Sample | null | undefined> {
        const samples = await this.samples(document);
        if (samples.length <= 1) {
            return samples[0] ?? null;
        }
        const picked = await vscode.window.showQuickPick(
            samples.map(sample => ({ label: sample.label, sample })),
            { placeHolder: 'Run on which sample?' },
        );
        return picked?.sample;
    }

    /** The document's samples in run order, as `cpl run --samples --list` names them. */
    private async samples(document: vscode.TextDocument | undefined): Promise<Sample[]> {
        const binary = serverBinary();
        if (!binary || document?.uri.scheme !== 'file') {
            return [];
        }
        const file = document.uri.fsPath;
        const result = await runCpl(binary, ['run', file, '--samples', '--list', '--format', 'json'], { cwd: path.dirname(file) });
        try {
            const listed: { name: string; input: string }[] = JSON.parse(result.stdout).samples;
            return listed.map(s => ({ label: s.name, input: s.input }));
        } catch {
            return [];
        }
    }
}
