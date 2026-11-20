// The commands behind the lenses: run a file on its samples, stress it
// against its brute / checker / interactor, or trace it. Each command runs
// `cpl`, writes what happened to the output panel and updates the inline
// overlays.

import * as path from 'node:path';
import * as vscode from 'vscode';
import { serverBinary } from './client';
import { CplResult, EXIT_FAILED, EXIT_NOT_BUILT, runCpl } from './cpl';
import { samplesOf } from './lenses';
import { Overlays } from './overlays';
import {
    diffOutput,
    parseCliDiagnostics,
    parseRunReport,
    parseStressReport,
    renderCounterexample,
    RuntimeErrorOverlay,
} from './results';
import { Sample } from './samples';

type SampleOutcome = 'passed' | 'failed' | 'printed' | 'stopped';

export class Runner {
    /** Only one background live-value run at a time per document. */
    private readonly liveRuns = new Map<string, number>();
    private liveRunId = 0;

    constructor(
        private readonly output: vscode.OutputChannel,
        private readonly overlays: Overlays,
    ) {}

    async runSample(uri?: vscode.Uri, sample?: Sample): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        const chosen = sample ?? (await this.pickSample(document));
        if (chosen === undefined) {
            return;
        }
        this.output.show(true);
        await this.runOne(document, chosen ?? undefined, { showValues: true, showStatus: true });
    }

    async runAllSamples(uri?: vscode.Uri): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        const samples = await samplesOf(document);
        if (samples.length === 0) {
            void vscode.window.showInformationMessage(
                'No samples yet. Save an input as `i1` beside the file, place files in `<file>.tests/`, or add `@sample("in", "out")`.',
            );
            return;
        }
        this.output.show(true);
        this.overlays.setRuntimeError(document.uri, undefined);
        const counts: Record<SampleOutcome, number> = { passed: 0, failed: 0, printed: 0, stopped: 0 };
        const shown = this.liveSample(samples);
        const summaryLines: string[] = [];
        await vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: `CP: Running ${samples.length} samples...`,
                cancellable: true,
            },
            async (_progress, token) => {
                const controller = new AbortController();
                const cancelSub = token.onCancellationRequested(() => controller.abort());
                try {
                    for (const sample of samples) {
                        if (token.isCancellationRequested) {
                            counts.stopped += 1;
                            return;
                        }
                        const outcome = await this.runOne(document, sample, {
                            showValues: sample === shown,
                            showStatus: false,
                            signal: controller.signal,
                            onSummary: line => summaryLines.push(line),
                        });
                        counts[outcome] += 1;
                        if (outcome === 'stopped') {
                            return;
                        }
                    }
                } finally {
                    cancelSub.dispose();
                }
            },
        );
        if (counts.stopped > 0) {
            return;
        }
        const checked = counts.passed + counts.failed;
        const summary =
            checked > 0
                ? `${counts.passed} of ${checked} samples passed` +
                  (counts.printed > 0 ? `; ${counts.printed} had no expected output` : '')
                : `ran ${counts.printed} samples; none has an expected output`;
        this.output.appendLine(summary);
        this.overlays.setSampleStatus(document.uri, {
            line: 0,
            ok: counts.failed === 0,
            label: summary,
            detail: summaryLines.join('\n') || undefined,
        });
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
            this.overlays.setRuntimeError(document.uri, undefined);
            const msg =
                report.status === 'pass'
                    ? `stress: ${report.cases ?? 'all'} cases passed`
                    : 'stress: FAIL (the report has no counterexample)';
            this.output.appendLine(msg);
            this.overlays.setSampleStatus(document.uri, {
                line: 0,
                ok: report.status === 'pass',
                label: msg,
            });
            return;
        }
        this.output.appendLine(renderCounterexample(report.counterexample));
        this.overlays.setSampleStatus(document.uri, undefined);
        this.overlays.setCounterexample(document.uri, report.counterexample);
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
        const panel = vscode.window.createWebviewPanel(
            'cp.trace',
            `Trace: ${path.basename(document.uri.fsPath)}`,
            { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
            { enableScripts: true },
        );
        panel.webview.html = result.stdout;
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
        const result = await runCpl(binary, ['run', document.uri.fsPath, '--format', 'json'], {
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
            this.overlays.setLiveValues(document.uri, sample.label, report.values);
            if (report.error) {
                this.overlays.setRuntimeError(document.uri, {
                    line: report.error.line ?? 0,
                    message: report.error.message,
                    sample: sample.label,
                });
            } else {
                this.overlays.setRuntimeError(document.uri, undefined);
            }
        }
    }

    /** The sample whose values are shown inline: `cp.liveValues.input`, or the first. */
    private liveSample(samples: readonly Sample[]): Sample | undefined {
        const wanted = vscode.workspace.getConfiguration('cp').get<string>('liveValues.input', '');
        return wanted === '' ? samples[0] : samples.find(sample => sample.label === wanted);
    }

    private async runOne(
        document: vscode.TextDocument,
        sample: Sample | undefined,
        options: {
            showValues?: boolean;
            showStatus?: boolean;
            signal?: AbortSignal;
            onSummary?: (line: string) => void;
        } = {},
    ): Promise<SampleOutcome> {
        const showValues = options.showValues ?? true;
        const showStatus = options.showStatus ?? false;
        const label = sample?.label ?? 'no input';
        const statusLine = sample?.line ?? 0;
        const stdin = sample ? await this.read(document, sample.input) : '';
        let result = await this.cpl(
            document,
            ['run', document.uri.fsPath, '--format', 'json'],
            `cpl run (${label})`,
            stdin,
            { allowUsageFallback: true, allowRuntimeError: true, signal: options.signal },
        );
        // Fall back to plain `cpl run <file>` when this `cpl` build doesn't accept `--format json` yet.
        if (result && result.code === 2 && /takes a file|unknown option|--format/i.test(result.stderr)) {
            result = await this.cpl(document, ['run', document.uri.fsPath], `cpl run (${label})`, stdin, {
                allowRuntimeError: true,
                signal: options.signal,
            });
        }
        if (!result) {
            return 'stopped';
        }
        const report = parseRunReport(result.stdout);
        const stdout = report?.stdout ?? result.stdout;
        const time = `${report?.timeMs ?? result.timeMs} ms`;
        if (report && sample && showValues) {
            this.overlays.setLiveValues(document.uri, sample.label, report.values);
        }
        if (result.code !== 0 || report?.error) {
            const parsed = extractRuntimeError(report?.error, result.stderr, label);
            this.overlays.setRuntimeError(document.uri, parsed);
            const summary = `${label}: runtime error (${time})`;
            this.output.appendLine(summary);
            this.output.appendLine(indent(report?.error?.message ?? result.stderr));
            options.onSummary?.(`${summary}: ${parsed.message}`);
            if (showStatus) {
                this.overlays.setSampleStatus(document.uri, {
                    line: statusLine,
                    ok: false,
                    label: `${label}: runtime error (${time})`,
                    detail: report?.error?.message ?? result.stderr.trimEnd(),
                });
            }
            return 'failed';
        }
        if (showStatus) {
            this.overlays.setRuntimeError(document.uri, undefined);
        }
        if (!sample?.expected) {
            const summary = `${label}: ${time}`;
            this.output.appendLine(summary);
            this.output.appendLine(indent(stdout));
            options.onSummary?.(summary);
            if (showStatus) {
                this.overlays.setSampleStatus(document.uri, {
                    line: statusLine,
                    ok: true,
                    label: summary,
                    detail: stdout.trimEnd() || '(empty output)',
                });
            }
            return 'printed';
        }
        const diff = diffOutput(await this.read(document, sample.expected), stdout);
        if (diff.same) {
            const summary = `${label}: passed (${time})`;
            this.output.appendLine(summary);
            options.onSummary?.(summary);
            if (showStatus) {
                this.overlays.setSampleStatus(document.uri, {
                    line: statusLine,
                    ok: true,
                    label: summary,
                    detail: stdout.trimEnd() || '(empty output)',
                });
            }
            return 'passed';
        }
        const failSummary = `${label}: FAILED at output line ${diff.line! + 1} (${time})`;
        const expectedLine = `expected: ${diff.expected ?? '(no more lines)'}`;
        const actualLine = `actual:   ${diff.actual ?? '(no more lines)'}`;
        this.output.appendLine(failSummary);
        this.output.appendLine(`  ${expectedLine}`);
        this.output.appendLine(`  ${actualLine}`);
        options.onSummary?.(`${failSummary} (${expectedLine}, ${actualLine})`);
        if (showStatus) {
            this.overlays.setSampleStatus(document.uri, {
                line: statusLine,
                ok: false,
                label: failSummary,
                detail: `${expectedLine}\n${actualLine}`,
            });
        }
        return 'failed';
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
                "Couldn't find `cpl`. Build it with `cargo build --release -p cpl-cli`, or set `cp.server.path`.",
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
            this.output.appendLine(indent(result.stderr));
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

function extractRuntimeError(
    reportError: { line?: number; message: string } | undefined,
    stderr: string,
    sample: string,
): RuntimeErrorOverlay {
    if (reportError) {
        return {
            line: reportError.line ?? 0,
            message: reportError.message,
            sample,
        };
    }
    const diagnostics = parseCliDiagnostics(stderr);
    const first = diagnostics.find(d => d.severity === 'error') ?? diagnostics[0];
    if (first) {
        return {
            line: first.line ?? 0,
            col: first.col,
            code: first.code,
            message: first.message,
            help: first.help,
            sample,
        };
    }
    const firstLine = stderr
        .trim()
        .split(/\r?\n/)
        .find(line => line.trim() !== '');
    return {
        line: 0,
        message: firstLine ?? 'runtime error',
        sample,
    };
}

function indent(text: string): string {
    return text
        .trimEnd()
        .split('\n')
        .map(line => '  ' + line)
        .join('\n');
}
