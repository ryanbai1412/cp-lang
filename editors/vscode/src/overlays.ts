// Draws the `view` a `cpl ... --format json` report carries, as is: one per
// document, until the next report or an edit. Decorations only.

import * as vscode from 'vscode';
import { clip, View } from './results';

function after(color: string, background?: string, fontStyle?: string): vscode.TextEditorDecorationType {
    return vscode.window.createTextEditorDecorationType({
        isWholeLine: background !== undefined,
        backgroundColor: background && new vscode.ThemeColor(background),
        overviewRulerColor: background && new vscode.ThemeColor(color),
        overviewRulerLane: vscode.OverviewRulerLane.Right,
        after: { color: new vscode.ThemeColor(color), margin: '0 0 0 3ch', fontStyle },
    });
}

export class Overlays implements vscode.Disposable {
    private readonly views = new Map<string, View>();
    private readonly problems = vscode.languages.createDiagnosticCollection('cp-runtime');
    /** By `view.lines[].kind`, and `ok`/`failed` for `view.status`. */
    private readonly types: Record<string, vscode.TextEditorDecorationType> = {
        error: after('cp.runtimeErrorForeground', 'cp.runtimeErrorBackground'),
        differs: after('cp.counterexampleForeground', 'cp.counterexampleBackground'),
        agrees: after('cp.counterexampleAgreeForeground'),
        input: after('cp.counterexampleForeground'),
        live: after('cp.liveValueForeground', undefined, 'italic'),
        ok: after('cp.counterexampleAgreeForeground'),
        failed: after('cp.runtimeErrorForeground'),
    };
    private readonly subscriptions = [
        vscode.window.onDidChangeVisibleTextEditors(() => this.renderAll()),
        // Marks belong to the text that was run, so an edit takes them away.
        vscode.workspace.onDidChangeTextDocument(e => e.contentChanges.length > 0 && this.views.has(e.document.uri.toString()) && this.show(e.document.uri, undefined)),
        vscode.workspace.onDidCloseTextDocument(document => this.show(document.uri, undefined)),
        vscode.workspace.onDidChangeConfiguration(e => e.affectsConfiguration('cp.liveValues') && this.renderAll()),
    ];

    /** Shows `view` on the document in place of what it showed; undefined clears it. */
    show(uri: vscode.Uri, view: View | undefined): void {
        const key = uri.toString();
        view ? this.views.set(key, view) : this.views.delete(key);
        const d = view?.diagnostic;
        const problem = d && new vscode.Diagnostic(new vscode.Range(d.line, d.col ?? 0, d.line, 1e6), d.help ? `${d.message} (${d.help})` : d.message);
        this.problems.set(uri, problem ? [Object.assign(problem, { source: d.source, code: d.code })] : []);
        this.renderAll();
    }

    dispose(): void {
        for (const item of [this.problems, ...Object.values(this.types), ...this.subscriptions]) {
            item.dispose();
        }
    }

    private renderAll(): void {
        const max = vscode.workspace.getConfiguration('cp').get<number>('liveValues.maxLength', 60);
        for (const editor of vscode.window.visibleTextEditors.filter(e => e.document.languageId === 'cp')) {
            const view = this.views.get(editor.document.uri.toString());
            const marks: { line: number; kind: string; label: string; hover?: string }[] = [...(view?.lines ?? [])];
            const s = view?.status;
            // A status draws only on a line with no mark.
            if (s && !marks.some(mark => mark.line === s.line)) {
                const hover = s.detail ? `${s.label}\n\n\`\`\`text\n${s.detail}\n\`\`\`` : s.label;
                marks.push({ line: s.line, kind: s.ok ? 'ok' : 'failed', label: `${s.ok ? '\u2713' : '\u2717'} ${s.label}`, hover });
            }
            for (const [kind, type] of Object.entries(this.types)) {
                const shown = marks.filter(mark => mark.kind === kind && mark.line < editor.document.lineCount);
                editor.setDecorations(type, shown.map(mark => {
                    const end = editor.document.lineAt(mark.line).range.end;
                    // The editor collapses runs of spaces in decoration text; non-breaking ones keep the columns.
                    const contentText = clip(mark.label, max).replace(/ /g, '\u00a0');
                    return { range: new vscode.Range(end, end), hoverMessage: new vscode.MarkdownString(mark.hover), renderOptions: { after: { contentText } } };
                }));
            }
        }
    }
}
