// Inline overlays: the `view` each `cpl ... --format json` report carries,
// drawn as is. Each report fills a slot (`live`, `error`, `stress`, ...); on
// a line the strongest mark wins, and the latest status and diagnostic.
// All decorations; none edits the document.

import * as vscode from 'vscode';
import {
    clip,
    Counterexample,
    counterexampleSummary,
    divergenceLabel,
    divergenceTitle,
    firstDifference,
    SampleStatusOverlay,
    stageLabel,
    View,
    VIEW_KINDS,
} from './results';

const KINDS = VIEW_KINDS;
type ViewKind = (typeof KINDS)[number];

const AFTER_MARGIN = '0 0 0 3ch';

function after(color: string, wholeLine = false): vscode.TextEditorDecorationType {
    return vscode.window.createTextEditorDecorationType({
        isWholeLine: wholeLine,
        backgroundColor: wholeLine ? new vscode.ThemeColor(color.replace('Foreground', 'Background')) : undefined,
        overviewRulerColor: wholeLine ? new vscode.ThemeColor(color) : undefined,
        overviewRulerLane: wholeLine ? vscode.OverviewRulerLane.Right : undefined,
        after: { color: new vscode.ThemeColor(color), margin: AFTER_MARGIN },
    });
}

export class Overlays implements vscode.Disposable {
    /** Per document, its views by slot, the latest set last. */
    private readonly byDocument = new Map<string, Map<string, View>>();
    private readonly diagnostics = vscode.languages.createDiagnosticCollection('cp-runtime');
    private readonly types: Record<ViewKind, vscode.TextEditorDecorationType> = {
        error: after('cp.runtimeErrorForeground', true),
        differs: after('cp.counterexampleForeground', true),
        agrees: after('cp.counterexampleAgreeForeground'),
        input: after('cp.counterexampleForeground'),
        live: vscode.window.createTextEditorDecorationType({
            after: { color: new vscode.ThemeColor('cp.liveValueForeground'), margin: AFTER_MARGIN, fontStyle: 'italic' },
            rangeBehavior: vscode.DecorationRangeBehavior.ClosedOpen,
        }),
    };
    private readonly passType = after('cp.counterexampleAgreeForeground');
    private readonly failType = after('cp.runtimeErrorForeground');
    private readonly subscriptions: vscode.Disposable[];

    constructor() {
        this.subscriptions = [
            vscode.window.onDidChangeVisibleTextEditors(() => this.renderAll()),
            // Marks belong to the text that was run. Once it changes they
            // may sit on the wrong lines, so they go until the next run.
            vscode.workspace.onDidChangeTextDocument(event => {
                if (event.contentChanges.length > 0 && this.byDocument.has(event.document.uri.toString())) {
                    this.clear(event.document.uri);
                }
            }),
            vscode.workspace.onDidCloseTextDocument(document => this.clear(document.uri)),
            vscode.workspace.onDidChangeConfiguration(event => {
                if (event.affectsConfiguration('cp.liveValues')) {
                    this.renderAll();
                }
            }),
        ];
    }

    /** Shows `view` in `slot` of the document, replacing what the slot showed; undefined empties it. */
    setView(uri: vscode.Uri, slot: string, view: View | undefined): void {
        const key = uri.toString();
        const views = this.byDocument.get(key) ?? new Map<string, View>();
        views.delete(slot);
        if (view) {
            views.set(slot, view);
        }
        this.byDocument.set(key, views);
        this.syncDiagnostic(uri);
        this.renderAll();
    }

    /** Until `cpl run`'s sample verdicts come back as a view. */
    setSampleStatus(uri: vscode.Uri, status: SampleStatusOverlay | undefined): void {
        this.setView(uri, 'status', status && { panel: '', status });
    }

    /** Until `cpl stress` reports its marks as a view. */
    setCounterexample(uri: vscode.Uri, counterexample: Counterexample | undefined): void {
        this.setView(uri, 'stress', counterexample && counterexampleView(counterexample));
    }

    clear(uri: vscode.Uri): void {
        this.byDocument.delete(uri.toString());
        this.diagnostics.delete(uri);
        this.renderAll();
    }

    dispose(): void {
        for (const item of [this.diagnostics, ...Object.values(this.types), this.passType, this.failType, ...this.subscriptions]) {
            item.dispose();
        }
    }

    private views(uri: vscode.Uri): View[] {
        return [...(this.byDocument.get(uri.toString())?.values() ?? [])];
    }

    private syncDiagnostic(uri: vscode.Uri): void {
        const d = this.views(uri).reverse().find(view => view.diagnostic)?.diagnostic;
        const doc = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri.toString());
        if (!d) {
            this.diagnostics.delete(uri);
            return;
        }
        let range = new vscode.Range(d.line, d.col ?? 0, d.line, (d.col ?? 0) + 1);
        if (doc && d.line < doc.lineCount) {
            const text = doc.lineAt(d.line);
            const start = Math.min(d.col ?? text.firstNonWhitespaceCharacterIndex, text.text.length);
            range = new vscode.Range(d.line, start, d.line, Math.max(start + 1, text.text.length));
        }
        const diag = new vscode.Diagnostic(range, d.help ? `${d.message} (${d.help})` : d.message, vscode.DiagnosticSeverity.Error);
        diag.source = d.source;
        diag.code = d.code;
        this.diagnostics.set(uri, [diag]);
    }

    private renderAll(): void {
        for (const editor of vscode.window.visibleTextEditors) {
            if (editor.document.languageId === 'cp') {
                this.render(editor);
            }
        }
    }

    private render(editor: vscode.TextEditor): void {
        const views = this.views(editor.document.uri);
        const max = vscode.workspace.getConfiguration('cp').get<number>('liveValues.maxLength', 60);
        const lineCount = editor.document.lineCount;
        const decoration = (line: number, text: string, hover: vscode.MarkdownString): vscode.DecorationOptions => {
            const end = editor.document.lineAt(line).range.end;
            // The editor collapses runs of spaces in decoration text; non-breaking ones keep the columns.
            return { range: new vscode.Range(end, end), renderOptions: { after: { contentText: text.replace(/ /g, '\u00a0') } }, hoverMessage: hover };
        };
        const marks = new Map<number, NonNullable<View['lines']>[number]>();
        for (const mark of views.flatMap(view => view.lines ?? [])) {
            const held = marks.get(mark.line);
            if (mark.line < lineCount && (!held || KINDS.indexOf(mark.kind) <= KINDS.indexOf(held.kind))) {
                marks.set(mark.line, mark);
            }
        }
        for (const kind of KINDS) {
            const shown = [...marks.values()].filter(mark => mark.kind === kind);
            editor.setDecorations(
                this.types[kind],
                shown.map(mark => decoration(mark.line, clip(mark.label, max), new vscode.MarkdownString(mark.hover ?? ''))),
            );
        }
        const status = views.reverse().find(view => view.status)?.status;
        const badge = status && status.line < lineCount && !marks.has(status.line) ? status : undefined;
        const hover = new vscode.MarkdownString(badge?.label);
        if (badge?.detail) {
            hover.appendCodeblock(badge.detail, 'text');
        }
        const badges = badge ? [decoration(badge.line, `${badge.ok ? '\u2713' : '\u2717'} ${clip(badge.label, max)}`, hover)] : [];
        editor.setDecorations(this.passType, badge?.ok ? badges : []);
        editor.setDecorations(this.failType, badge?.ok ? [] : badges);
    }
}

function codeBlock(text: string, lang: string): string {
    return `\n\n\`\`\`${lang}\n${text}\n\`\`\``;
}

/** The counterexample's marks as `cpl stress`'s view will carry them. */
function counterexampleView(c: Counterexample): View {
    const lines: NonNullable<View['lines']> = [];
    const first = firstDifference(c);
    for (const stage of c.stages) {
        if (stage.line !== undefined) {
            const hover = `\`${stage.name}\` on the counterexample:` + codeBlock(`${c.input}\nfast  = ${stage.fast}\nbrute = ${stage.brute}`, 'cp');
            // Later differences usually follow from the first, so only the first is marked.
            lines.push({ line: stage.line, kind: stage === first ? 'differs' : 'agrees', label: stageLabel(stage, Infinity), hover });
        }
    }
    const d = c.divergence;
    const at = d?.fast?.line;
    if (d && at !== undefined && !lines.some(l => l.line === at)) {
        const steps = d.inside.map(step => `line ${step.line !== undefined ? step.line + 1 : '?'}: ${step.locals}`);
        const body = [c.input, `fast  = ${d.fast?.value}`, `brute = ${d.brute?.value ?? 'nothing'}`, ...steps].join('\n');
        lines.push({ line: at, kind: 'differs', label: divergenceLabel(d, Infinity), hover: divergenceTitle(d) + codeBlock(body, 'cp') });
    }
    if (!lines.some(l => l.line === 0)) {
        let hover = 'The smallest input `cpl stress` found where the fast code and the reference disagree.';
        if (c.saved) {
            hover += ` Saved as \`${c.saved}\`.`;
        }
        if (c.stages.length === 0 && (c.got !== undefined || c.expected !== undefined)) {
            const details = [
                c.rawInput?.trimEnd() || c.input,
                c.got !== undefined ? `fast  = ${c.got.trimEnd()}` : undefined,
                c.expected !== undefined ? `brute = ${c.expected.trimEnd()}` : undefined,
            ].filter(part => part !== undefined && part !== '');
            hover += details.length > 0 ? codeBlock(details.join('\n'), 'cp') : '';
        }
        lines.push({ line: 0, kind: 'input', label: counterexampleSummary(c, Infinity), hover });
    }
    return { panel: '', lines };
}
