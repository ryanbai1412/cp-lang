// The trace of a run, beside its file: one panel per file, reused by the
// next trace. The editor follows the step the viewer shows, and a click on
// a line in the viewer's source jumps to it.

import * as path from 'node:path';
import * as vscode from 'vscode';
import { parseTraceMessage } from './results';

export class TraceView implements vscode.Disposable {
    private readonly panels = new Map<string, vscode.WebviewPanel>();
    private readonly stepLine = vscode.window.createTextEditorDecorationType({
        isWholeLine: true,
        backgroundColor: new vscode.ThemeColor('editor.rangeHighlightBackground'),
        overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.rangeHighlightForeground'),
        overviewRulerLane: vscode.OverviewRulerLane.Full,
    });

    show(document: vscode.TextDocument, html: string): vscode.WebviewPanel {
        const key = document.uri.toString();
        let panel = this.panels.get(key);
        if (panel) {
            panel.reveal(vscode.ViewColumn.Beside, true);
        } else {
            panel = vscode.window.createWebviewPanel(
                'cp.trace',
                `Trace: ${path.basename(document.uri.fsPath)}`,
                { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
                { enableScripts: true, retainContextWhenHidden: true },
            );
            this.panels.set(key, panel);
            panel.webview.onDidReceiveMessage(message => void this.follow(document.uri, message));
            panel.onDidDispose(() => {
                this.panels.delete(key);
                this.editorsOf(document.uri).forEach(editor => editor.setDecorations(this.stepLine, []));
            });
        }
        panel.webview.html = html;
        return panel;
    }

    private async follow(uri: vscode.Uri, raw: unknown): Promise<void> {
        const message = parseTraceMessage(raw);
        if (!message) {
            return;
        }
        let editors = this.editorsOf(uri);
        if (message.type === 'reveal' || editors.length === 0) {
            const document = await vscode.workspace.openTextDocument(uri);
            const column = editors[0]?.viewColumn ?? vscode.ViewColumn.One;
            editors = [await vscode.window.showTextDocument(document, { viewColumn: column, preserveFocus: message.type === 'step' })];
        }
        for (const editor of editors) {
            const line = Math.min(message.line - 1, editor.document.lineCount - 1);
            const range = editor.document.lineAt(line).range;
            editor.setDecorations(this.stepLine, [range]);
            editor.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
            if (message.type === 'reveal') {
                editor.selection = new vscode.Selection(range.start, range.start);
            }
        }
    }

    private editorsOf(uri: vscode.Uri): readonly vscode.TextEditor[] {
        return vscode.window.visibleTextEditors.filter(editor => editor.document.uri.toString() === uri.toString());
    }

    dispose(): void {
        this.panels.forEach(panel => panel.dispose());
        this.stepLine.dispose();
    }
}
