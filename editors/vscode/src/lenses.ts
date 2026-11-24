// Replay lenses while a debug session is stopped. The Run, Trace, Test and
// Stress lenses come from the language server (`cpl.exec`).

import * as vscode from 'vscode';
import { replayLenses } from './samples';

export class LensProvider implements vscode.CodeLensProvider, vscode.Disposable {
    private readonly changed = new vscode.EventEmitter<void>();
    readonly onDidChangeCodeLenses = this.changed.event;
    private readonly subscriptions: vscode.Disposable[];

    constructor() {
        this.subscriptions = [
            vscode.debug.onDidStartDebugSession(() => this.changed.fire()),
            vscode.debug.onDidTerminateDebugSession(() => this.changed.fire()),
            vscode.workspace.onDidChangeConfiguration(event => {
                if (event.affectsConfiguration('cp.codeLens')) {
                    this.changed.fire();
                }
            }),
            this.changed,
        ];
    }

    async provideCodeLenses(document: vscode.TextDocument): Promise<vscode.CodeLens[]> {
        if (!vscode.workspace.getConfiguration('cp').get<boolean>('codeLens.enabled', true)) {
            return [];
        }
        const specs = vscode.debug.activeDebugSession?.type === 'cp' ? replayLenses(document.getText()) : [];
        return specs.map(spec => {
            const line = Math.min(spec.line, document.lineCount - 1);
            return new vscode.CodeLens(new vscode.Range(line, 0, line, 0), {
                title: spec.title,
                command: spec.command,
                arguments: [document.uri, ...spec.args],
            });
        });
    }

    dispose(): void {
        for (const item of this.subscriptions) {
            item.dispose();
        }
    }
}
