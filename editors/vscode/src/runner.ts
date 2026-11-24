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
import { samplesOf } from './lenses';
import { Overlays } from './overlays';
import {
    parseRunReport,
    parseStressReport,
    viewOf,
    renderCounterexample,
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

    async runSample(uri?: vscode.Uri, sample?: Sample): Promise<void> {
        const document = await this.document(uri);
        const chosen = document && (sample ?? (await this.pickSample(document)));
        if (document && chosen !== undefined) {
            await this.runSamples(document, chosen ? ['--case', chosen.label] : []);
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
        this.overlays.setView(document.uri, 'live', undefined);
        this.overlays.setView(document.uri, 'status', view);
    }

    async stress(uri?: vscode.Uri, part?: string): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        this.output.show(true);
        const extra = vscode.workspace.getConfiguration('cp').get<string[]>('stress.args', []);
        const baseArgs = ['stress', document.uri.fsPath, '--format', 'json', ...extra];
        const result = await vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: part ? `CP: Stressing ${part}...` : `CP: Stressing ${path.basename(document.uri.fsPath)}...`,
                cancellable: true,
            },
            async (_progress, token) => {
                const controller = new AbortController();
                const cancelSub = token.onCancellationRequested(() => controller.abort());
                try {
                    let run = await this.cpl(
                        document,
                        part ? [...baseArgs, '--part', part] : baseArgs,
                        'cpl stress',
                        undefined,
                        { allowUsageFallback: Boolean(part), signal: controller.signal },
                    );
                    // Current `cpl stress` does not take `--part`; retry whole-file stress cleanly.
                    if (part && run && run.code === 2 && /--part/.test(run.stderr)) {
                        run = await this.cpl(document, baseArgs, 'cpl stress', undefined, {
                            signal: controller.signal,
                        });
                    }
                    return run;
                } finally {
                    cancelSub.dispose();
                }
            },
        );
        if (!result) {
            return;
        }
        const report = parseStressReport(result.stdout);
        if (!report) {
            this.printRaw(result);
            return;
        }
        if (report.status === 'pass' || !report.counterexample) {
            this.overlays.setCounterexample(document.uri, undefined);
            this.overlays.setView(document.uri, 'error', undefined);
            const msg =
                report.status === 'pass'
                    ? `stress: ${report.cases ?? 'all'} cases passed`
                    : 'stress: FAIL (the report has no counterexample)';
            this.output.appendLine(msg);
            this.overlays.setView(document.uri, 'status', { panel: '', status: { line: 0, ok: report.status === 'pass', label: msg } });
            return;
        }
        this.output.appendLine(renderCounterexample(report.counterexample));
        this.overlays.setView(document.uri, 'status', undefined);
        this.overlays.setCounterexample(document.uri, report.counterexample);
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
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        this.output.show(true);
        const copy = vscode.workspace.getConfiguration('cp').get<boolean>('submit.copy', true);
        const args = ['submit', document.uri.fsPath, '--json', ...(copy ? ['--copy'] : [])];
        const result = await this.cpl(document, args, 'cpl submit');
        if (!result) {
            return;
        }
        const view = viewOf(result.stdout);
        if (!view?.status) {
            this.printRaw(result);
            return;
        }
        this.output.append(view.panel);
        this.overlays.setView(document.uri, 'status', view);
        if (view.status.ok) {
            void vscode.window.showInformationMessage(view.status.label);
        } else {
            void vscode.window.showWarningMessage(view.status.label);
        }
    }

    async trace(uri?: vscode.Uri, sample?: Sample, part?: string): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        const chosen = sample ?? (await this.pickSample(document));
        if (chosen === undefined) {
            return;
        }
        const args = ['run', document.uri.fsPath, '--trace', ...(part ? [part] : []), '--format', 'html'];
        const stdin = chosen ? await this.read(document, chosen.input) : '';
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
        const sample = this.liveSample(await samplesOf(document));
        if (!sample) {
            return;
        }
        const key = document.uri.toString();
        const id = ++this.liveRunId;
        this.liveRuns.set(key, id);
        const version = document.version;
        const result = await runCpl(binary, ['run', document.uri.fsPath, '--format', 'json', '--sample', sample.label], {
            cwd: path.dirname(document.uri.fsPath),
            stdin: await this.read(document, sample.input),
            timeoutMs: config.get<number>('run.timeoutMs', 10000),
        });
        // A newer save or an edit since the run started makes this result stale.
        if (this.liveRuns.get(key) !== id || document.version !== version) {
            return;
        }
        const report = parseRunReport(result.stdout);
        if (report) {
            this.overlays.setView(document.uri, 'error', undefined);
            this.overlays.setView(document.uri, 'live', report.view);
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
        options?: { allowUsageFallback?: boolean; allowRuntimeError?: boolean; signal?: AbortSignal },
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
        if (options?.allowUsageFallback && result.code === 2 && result.stdout.trim() === '') {
            return result;
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
        const samples = await samplesOf(document);
        if (samples.length <= 1) {
            return samples[0] ?? null;
        }
        const picked = await vscode.window.showQuickPick(
            samples.map(sample => ({ label: sample.label, sample })),
            { placeHolder: 'Run on which sample?' },
        );
        return picked?.sample;
    }

    private async read(document: vscode.TextDocument, source: { text: string } | { file: string }): Promise<string> {
        if ('text' in source) {
            return source.text;
        }
        const file = vscode.Uri.file(path.join(path.dirname(document.uri.fsPath), source.file));
        return Buffer.from(await vscode.workspace.fs.readFile(file)).toString('utf8');
    }
}
