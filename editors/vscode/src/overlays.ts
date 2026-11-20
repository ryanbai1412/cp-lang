// Inline overlays: the value of each top-level `let` on a sample, shown at
// the end of its line, and a failing stress case laid over the parts that
// produced it. Both are decorations; neither edits the document.

import * as vscode from 'vscode';
import {
    Counterexample,
    counterexampleSummary,
    firstDifference,
    liveLabel,
    LiveValue,
    RuntimeErrorOverlay,
    runtimeErrorLabel,
    SampleStatusOverlay,
    sampleStatusLabel,
    stageLabel,
} from './results';

interface DocumentOverlay {
    live?: { sample: string; values: LiveValue[] };
    counterexample?: Counterexample;
    runtimeError?: RuntimeErrorOverlay;
    status?: SampleStatusOverlay;
}

const AFTER_MARGIN = '0 0 0 3ch';

export class Overlays implements vscode.Disposable {
    private readonly byDocument = new Map<string, DocumentOverlay>();
    private readonly runtimeDiagnostics = vscode.languages.createDiagnosticCollection('cp-runtime');
    private readonly liveType = vscode.window.createTextEditorDecorationType({
        after: { color: new vscode.ThemeColor('cp.liveValueForeground'), margin: AFTER_MARGIN, fontStyle: 'italic' },
        rangeBehavior: vscode.DecorationRangeBehavior.ClosedOpen,
    });
    private readonly differsType = vscode.window.createTextEditorDecorationType({
        isWholeLine: true,
        backgroundColor: new vscode.ThemeColor('cp.counterexampleBackground'),
        overviewRulerColor: new vscode.ThemeColor('cp.counterexampleForeground'),
        overviewRulerLane: vscode.OverviewRulerLane.Right,
        after: { color: new vscode.ThemeColor('cp.counterexampleForeground'), margin: AFTER_MARGIN },
    });
    private readonly agreesType = vscode.window.createTextEditorDecorationType({
        after: { color: new vscode.ThemeColor('cp.counterexampleAgreeForeground'), margin: AFTER_MARGIN },
    });
    private readonly inputType = vscode.window.createTextEditorDecorationType({
        after: { color: new vscode.ThemeColor('cp.counterexampleForeground'), margin: AFTER_MARGIN },
    });
    private readonly runtimeErrorType = vscode.window.createTextEditorDecorationType({
        isWholeLine: true,
        backgroundColor: new vscode.ThemeColor('cp.runtimeErrorBackground'),
        overviewRulerColor: new vscode.ThemeColor('cp.runtimeErrorForeground'),
        overviewRulerLane: vscode.OverviewRulerLane.Right,
        after: { color: new vscode.ThemeColor('cp.runtimeErrorForeground'), margin: AFTER_MARGIN },
    });
    private readonly statusPassType = vscode.window.createTextEditorDecorationType({
        after: { color: new vscode.ThemeColor('cp.counterexampleAgreeForeground'), margin: AFTER_MARGIN },
    });
    private readonly statusFailType = vscode.window.createTextEditorDecorationType({
        after: { color: new vscode.ThemeColor('cp.runtimeErrorForeground'), margin: AFTER_MARGIN },
    });
    private readonly subscriptions: vscode.Disposable[];

    constructor() {
        this.subscriptions = [
            vscode.window.onDidChangeVisibleTextEditors(() => this.renderAll()),
            // Values belong to the text that was run. Once it changes they
            // may sit on the wrong lines, so they go until the next run.
            vscode.workspace.onDidChangeTextDocument(event => {
                if (event.contentChanges.length > 0) {
                    const removed = this.byDocument.delete(event.document.uri.toString());
                    this.runtimeDiagnostics.delete(event.document.uri);
                    if (removed) {
                        this.renderAll();
                    }
                }
            }),
            vscode.workspace.onDidCloseTextDocument(document => {
                this.byDocument.delete(document.uri.toString());
                this.runtimeDiagnostics.delete(document.uri);
            }),
            vscode.workspace.onDidChangeConfiguration(event => {
                if (event.affectsConfiguration('cp.liveValues')) {
                    this.renderAll();
                }
            }),
        ];
    }

    setLiveValues(uri: vscode.Uri, sample: string, values: LiveValue[]): void {
        this.entry(uri).live = { sample, values };
        this.renderAll();
    }

    setCounterexample(uri: vscode.Uri, counterexample: Counterexample | undefined): void {
        this.entry(uri).counterexample = counterexample;
        this.renderAll();
    }

    setRuntimeError(uri: vscode.Uri, error: RuntimeErrorOverlay | undefined): void {
        this.entry(uri).runtimeError = error;
        this.syncRuntimeDiagnostic(uri, error);
        this.renderAll();
    }

    setSampleStatus(uri: vscode.Uri, status: SampleStatusOverlay | undefined): void {
        this.entry(uri).status = status;
        this.renderAll();
    }

    clear(uri: vscode.Uri): void {
        this.byDocument.delete(uri.toString());
        this.runtimeDiagnostics.delete(uri);
        this.renderAll();
    }

    dispose(): void {
        for (const item of [
            this.runtimeDiagnostics,
            this.liveType,
            this.differsType,
            this.agreesType,
            this.inputType,
            this.runtimeErrorType,
            this.statusPassType,
            this.statusFailType,
            ...this.subscriptions,
        ]) {
            item.dispose();
        }
    }

    private syncRuntimeDiagnostic(uri: vscode.Uri, error: RuntimeErrorOverlay | undefined): void {
        if (!error) {
            this.runtimeDiagnostics.delete(uri);
            return;
        }
        const doc = vscode.workspace.textDocuments.find(d => d.uri.toString() === uri.toString());
        const line = doc ? Math.min(Math.max(0, error.line), Math.max(0, doc.lineCount - 1)) : Math.max(0, error.line);
        let range: vscode.Range;
        if (doc && doc.lineCount > 0) {
            const textLine = doc.lineAt(line);
            const startChar =
                error.col !== undefined
                    ? Math.min(Math.max(0, error.col), textLine.text.length)
                    : textLine.firstNonWhitespaceCharacterIndex;
            range = new vscode.Range(line, startChar, line, Math.max(startChar + 1, textLine.text.length));
        } else {
            const col = Math.max(0, error.col ?? 0);
            range = new vscode.Range(line, col, line, col + 1);
        }
        const fullMessage = error.help ? `${error.message} (${error.help})` : error.message;
        const diag = new vscode.Diagnostic(range, fullMessage, vscode.DiagnosticSeverity.Error);
        diag.source = error.sample ? `cpl (${error.sample})` : 'cpl (runtime)';
        if (error.code) {
            diag.code = error.code;
        }
        this.runtimeDiagnostics.set(uri, [diag]);
    }

    private entry(uri: vscode.Uri): DocumentOverlay {
        const key = uri.toString();
        let entry = this.byDocument.get(key);
        if (!entry) {
            entry = {};
            this.byDocument.set(key, entry);
        }
        return entry;
    }

    private renderAll(): void {
        for (const editor of vscode.window.visibleTextEditors) {
            if (editor.document.languageId === 'cp') {
                this.render(editor);
            }
        }
    }

    private render(editor: vscode.TextEditor): void {
        const overlay = this.byDocument.get(editor.document.uri.toString()) ?? {};
        const max = vscode.workspace.getConfiguration('cp').get<number>('liveValues.maxLength', 60);
        const lineCount = editor.document.lineCount;
        const endOf = (line: number) => editor.document.lineAt(line).range.end;
        const after = (line: number, text: string, hover: vscode.MarkdownString): vscode.DecorationOptions => ({
            range: new vscode.Range(endOf(line), endOf(line)),
            // The editor collapses runs of spaces in decoration text; non-breaking ones keep the columns.
            renderOptions: { after: { contentText: text.replace(/ /g, '\u00a0') } },
            hoverMessage: hover,
        });

        const runtimeErrors: vscode.DecorationOptions[] = [];
        const occupiedLines = new Set<number>();
        if (overlay.runtimeError && lineCount > 0) {
            const targetLine = Math.min(Math.max(0, overlay.runtimeError.line), lineCount - 1);
            occupiedLines.add(targetLine);
            const title = overlay.runtimeError.sample
                ? `Runtime error on \`${overlay.runtimeError.sample}\`:`
                : 'Runtime error:';
            const hover = new vscode.MarkdownString(title);
            const body = overlay.runtimeError.help
                ? `${overlay.runtimeError.message}\nhelp: ${overlay.runtimeError.help}`
                : overlay.runtimeError.message;
            hover.appendCodeblock(body, 'text');
            runtimeErrors.push(after(targetLine, runtimeErrorLabel(overlay.runtimeError, max), hover));
        }
        editor.setDecorations(this.runtimeErrorType, runtimeErrors);

        const counterexample = overlay.counterexample;
        const stageLines = new Set(counterexample?.stages.map(stage => stage.line));
        const live: vscode.DecorationOptions[] = [];
        const liveLines = new Set<number>();
        if (overlay.live) {
            const byLine = new Map<number, LiveValue[]>();
            for (const value of overlay.live.values) {
                // A line that shows a counterexample or runtime error shows only that.
                if (value.line < lineCount && !stageLines.has(value.line) && !occupiedLines.has(value.line)) {
                    byLine.set(value.line, [...(byLine.get(value.line) ?? []), value]);
                }
            }
            for (const [line, values] of byLine) {
                liveLines.add(line);
                const hover = new vscode.MarkdownString(`On \`${overlay.live.sample}\`:`);
                hover.appendCodeblock(values.map(v => `${v.name} = ${v.value}`).join('\n'), 'cp');
                live.push(after(line, liveLabel(overlay.live.sample, values, max), hover));
            }
        }
        editor.setDecorations(this.liveType, live);

        const differs: vscode.DecorationOptions[] = [];
        const agrees: vscode.DecorationOptions[] = [];
        const input: vscode.DecorationOptions[] = [];
        if (counterexample) {
            const first = firstDifference(counterexample);
            for (const stage of counterexample.stages) {
                if (stage.line === undefined || stage.line >= lineCount) {
                    continue;
                }
                occupiedLines.add(stage.line);
                const hover = new vscode.MarkdownString(`\`${stage.name}\` on the counterexample:`);
                hover.appendCodeblock(`${counterexample.input}\nfast  = ${stage.fast}\nbrute = ${stage.brute}`, 'cp');
                // Later differences usually follow from the first, so only the first is marked.
                (stage === first ? differs : agrees).push(after(stage.line, stageLabel(stage, max), hover));
            }
            if (lineCount > 0 && !stageLines.has(0) && !occupiedLines.has(0)) {
                occupiedLines.add(0);
                const hover = new vscode.MarkdownString(
                    'The smallest input `cpl stress` found where the fast code and the reference disagree.',
                );
                if (counterexample.saved) {
                    hover.appendMarkdown(` Saved as \`${counterexample.saved}\`.`);
                }
                if (counterexample.stages.length === 0 && (counterexample.got !== undefined || counterexample.expected !== undefined)) {
                    const details = [
                        counterexample.rawInput?.trimEnd() || counterexample.input,
                        counterexample.got !== undefined ? `fast  = ${counterexample.got.trimEnd()}` : undefined,
                        counterexample.expected !== undefined ? `brute = ${counterexample.expected.trimEnd()}` : undefined,
                    ]
                        .filter((part): part is string => part !== undefined && part !== '')
                        .join('\n');
                    if (details) {
                        hover.appendCodeblock(details, 'cp');
                    }
                }
                input.push(after(0, counterexampleSummary(counterexample, max), hover));
            }
        }
        editor.setDecorations(this.differsType, differs);
        editor.setDecorations(this.agreesType, agrees);
        editor.setDecorations(this.inputType, input);

        const statusPass: vscode.DecorationOptions[] = [];
        const statusFail: vscode.DecorationOptions[] = [];
        if (overlay.status && lineCount > 0) {
            const targetLine = Math.min(Math.max(0, overlay.status.line), lineCount - 1);
            if (!occupiedLines.has(targetLine) && !liveLines.has(targetLine)) {
                const hover = new vscode.MarkdownString(overlay.status.label);
                if (overlay.status.detail) {
                    hover.appendCodeblock(overlay.status.detail, 'text');
                }
                const decoration = after(targetLine, sampleStatusLabel(overlay.status, max), hover);
                (overlay.status.ok ? statusPass : statusFail).push(decoration);
            }
        }
        editor.setDecorations(this.statusPassType, statusPass);
        editor.setDecorations(this.statusFailType, statusFail);
    }
}
