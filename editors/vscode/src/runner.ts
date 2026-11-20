// The commands behind the lenses: run a file on its samples, stress it
// against its brute, trace it. Each one runs `cpl`, writes what happened to
// the output panel and updates the inline overlays.

import * as path from 'node:path';
import * as vscode from 'vscode';
import { serverBinary } from './client';
import { CplResult, EXIT_NOT_BUILT, runCpl } from './cpl';
import { samplesOf } from './lenses';
import { Overlays } from './overlays';
import { diffOutput, parseRunReport, parseStressReport, renderCounterexample } from './results';
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
        await this.runOne(document, chosen ?? undefined);
    }

    async runAllSamples(uri?: vscode.Uri): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        const samples = await samplesOf(document);
        if (samples.length === 0) {
            void vscode.window.showInformationMessage(
                'No samples yet. Add `@sample("input", "output")` to the file, or save an input as `i1` beside it.',
            );
            return;
        }
        this.output.show(true);
        const counts: Record<SampleOutcome, number> = { passed: 0, failed: 0, printed: 0, stopped: 0 };
        const shown = this.liveSample(samples);
        for (const sample of samples) {
            const outcome = await this.runOne(document, sample, sample === shown);
            counts[outcome] += 1;
            if (outcome === 'stopped') {
                return;
            }
        }
        const checked = counts.passed + counts.failed;
        this.output.appendLine(
            checked > 0
                ? `${counts.passed} of ${checked} samples passed` +
                      (counts.printed > 0 ? `; ${counts.printed} had no expected output` : '')
                : `ran ${counts.printed} samples; none has an expected output`,
        );
    }

    async stress(uri?: vscode.Uri, part?: string): Promise<void> {
        const document = await this.document(uri);
        if (!document) {
            return;
        }
        this.output.show(true);
        const args = ['stress', document.uri.fsPath, '--format', 'json', ...(part ? ['--part', part] : [])];
        const result = await this.cpl(document, args, 'cpl stress');
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
            this.output.appendLine(
                report.status === 'pass'
                    ? `stress: ${report.cases ?? 'all'} cases agree with the brute`
                    : 'stress: FAIL (the report has no counterexample)',
            );
            return;
        }
        this.output.appendLine(renderCounterexample(report.counterexample));
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

    /** Recomputes the inline values of a saved file, quietly: nothing is shown on failure. */
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
        showValues = true,
    ): Promise<SampleOutcome> {
        const label = sample?.label ?? 'no input';
        const stdin = sample ? await this.read(document, sample.input) : '';
        const result = await this.cpl(document, ['run', document.uri.fsPath, '--format', 'json'], `cpl run (${label})`, stdin);
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
            this.output.appendLine(`${label}: runtime error (${time})`);
            this.output.appendLine(indent(report?.error?.message ?? result.stderr));
            return 'failed';
        }
        if (!sample?.expected) {
            this.output.appendLine(`${label}: ${time}`);
            this.output.appendLine(indent(stdout));
            return 'printed';
        }
        const diff = diffOutput(await this.read(document, sample.expected), stdout);
        if (diff.same) {
            this.output.appendLine(`${label}: passed (${time})`);
            return 'passed';
        }
        this.output.appendLine(`${label}: FAILED at output line ${diff.line! + 1} (${time})`);
        this.output.appendLine(`  expected: ${diff.expected ?? '(no more lines)'}`);
        this.output.appendLine(`  actual:   ${diff.actual ?? '(no more lines)'}`);
        return 'failed';
    }

    /**
     * Runs `cpl`. Returns undefined, after telling the user why, when the
     * command could not do its work at all: no binary, the source has errors,
     * or this part of the toolchain is not built yet.
     */
    private async cpl(
        document: vscode.TextDocument,
        args: string[],
        what: string,
        stdin?: string,
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
        });
        if (result.spawnError) {
            this.output.appendLine(`${what}: could not start ${binary}: ${result.spawnError}`);
            return undefined;
        }
        if (result.timedOut) {
            this.output.appendLine(`${what}: stopped after ${result.timeMs} ms (\`cp.run.timeoutMs\`)`);
            return undefined;
        }
        if (result.code === EXIT_NOT_BUILT) {
            // The file checked clean; this command's back end is not in this `cpl` yet.
            this.output.appendLine(result.stderr.trim());
            return undefined;
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

function indent(text: string): string {
    return text
        .trimEnd()
        .split('\n')
        .map(line => '  ' + line)
        .join('\n');
}
