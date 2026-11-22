// Run, Stress and Trace lenses. `samples.ts` decides which lenses a file
// gets; this file reads the sample files and hands the result to VS Code.

import * as path from 'node:path';
import * as vscode from 'vscode';
import { fileSamples, planLenses, replayLenses, Sample } from './samples';

async function names(folder: vscode.Uri): Promise<string[]> {
    try {
        const entries = await vscode.workspace.fs.readDirectory(folder);
        return entries.filter(([, type]) => type !== vscode.FileType.Directory).map(([name]) => name);
    } catch {
        return [];
    }
}

/** Every sample of a document: the files beside it (`i1`, `<stem>.tests/`, `<stem>.samples/`). */
export async function samplesOf(document: vscode.TextDocument): Promise<Sample[]> {
    if (document.uri.scheme !== 'file') {
        return [];
    }
    const folder = vscode.Uri.file(path.dirname(document.uri.fsPath));
    const stem = path.basename(document.uri.fsPath, '.cp');
    const [here, tests, samples] = await Promise.all([
        names(folder),
        names(vscode.Uri.joinPath(folder, `${stem}.tests`)),
        names(vscode.Uri.joinPath(folder, `${stem}.samples`)),
    ]);
    return fileSamples(stem, here, tests, samples);
}

export class LensProvider implements vscode.CodeLensProvider, vscode.Disposable {
    private readonly changed = new vscode.EventEmitter<void>();
    readonly onDidChangeCodeLenses = this.changed.event;
    private readonly subscriptions: vscode.Disposable[];

    constructor() {
        // A sample file appearing or going away changes the lenses of the source beside it.
        const watcher = vscode.workspace.createFileSystemWatcher('**/{i[0-9]*,o[0-9]*,*.in,*.out}');
        this.subscriptions = [
            watcher,
            watcher.onDidCreate(() => this.changed.fire()),
            watcher.onDidDelete(() => this.changed.fire()),
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
        const specs = planLenses(document.getText(), await samplesOf(document));
        if (vscode.debug.activeDebugSession?.type === 'cp') {
            specs.push(...replayLenses(document.getText()));
        }
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
