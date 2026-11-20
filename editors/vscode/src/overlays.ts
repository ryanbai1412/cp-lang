// Inline overlays: the value of each top-level `let` on a sample, shown at
// the end of its line, and a failing stress case laid over the parts that
// produced it. Both are decorations; neither edits the document.

import * as vscode from 'vscode';
import { Counterexample, firstDifference, liveLabel, LiveValue, stageLabel } from './results';

interface DocumentOverlay {
    live?: { sample: string; values: LiveValue[] };
    counterexample?: Counterexample;
}

const AFTER_MARGIN = '0 0 0 3ch';

export class Overlays implements vscode.Disposable {
    private readonly byDocument = new Map<string, DocumentOverlay>();
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
    private readonly subscriptions: vscode.Disposable[];

    constructor() {
        this.subscriptions = [
            vscode.window.onDidChangeVisibleTextEditors(() => this.renderAll()),
            // Values belong to the text that was run. Once it changes they
            // may sit on the wrong lines, so they go until the next run.
            vscode.workspace.onDidChangeTextDocument(event => {
                if (event.contentChanges.length > 0 && this.byDocument.delete(event.document.uri.toString())) {
                    this.renderAll();
                }
            }),
            vscode.workspace.onDidCloseTextDocument(document => this.byDocument.delete(document.uri.toString())),
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

    clear(uri: vscode.Uri): void {
        this.byDocument.delete(uri.toString());
        this.renderAll();
    }

    dispose(): void {
        for (const item of [this.liveType, this.differsType, this.agreesType, this.inputType, ...this.subscriptions]) {
            item.dispose();
        }
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

        const live: vscode.DecorationOptions[] = [];
        const counterexample = overlay.counterexample;
        const stageLines = new Set(counterexample?.stages.map(stage => stage.line));
        if (overlay.live) {
            const byLine = new Map<number, LiveValue[]>();
            for (const value of overlay.live.values) {
                // A line that shows the counterexample shows only that.
                if (value.line < lineCount && !stageLines.has(value.line)) {
                    byLine.set(value.line, [...(byLine.get(value.line) ?? []), value]);
                }
            }
            for (const [line, values] of byLine) {
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
                const hover = new vscode.MarkdownString(`\`${stage.name}\` on the counterexample:`);
                hover.appendCodeblock(`${counterexample.input}\nfast  = ${stage.fast}\nbrute = ${stage.brute}`, 'cp');
                // Later differences usually follow from the first, so only the first is marked.
                (stage === first ? differs : agrees).push(after(stage.line, stageLabel(stage, max), hover));
            }
            if (lineCount > 0 && !stageLines.has(0)) {
                const hover = new vscode.MarkdownString('The smallest input `cpl stress` found where the fast code and the brute disagree.');
                if (counterexample.saved) {
                    hover.appendMarkdown(` Saved as \`${counterexample.saved}\`.`);
                }
                input.push(after(0, `counterexample: ${counterexample.input}`, hover));
            }
        }
        editor.setDecorations(this.differsType, differs);
        editor.setDecorations(this.agreesType, agrees);
        editor.setDecorations(this.inputType, input);
    }
}
