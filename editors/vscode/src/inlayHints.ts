// VS Code fallback inlay hints provider for when `cpl lsp` does not
// advertise `inlayHintProvider`.

import * as vscode from 'vscode';
import { computeInlayHints } from './inlayHintsCore';

export { computeInlayHints, HintSpec } from './inlayHintsCore';

export function registerFallbackInlayHints(selector: vscode.DocumentSelector): vscode.Disposable {
    const emitter = new vscode.EventEmitter<void>();
    const configWatcher = vscode.workspace.onDidChangeConfiguration(event => {
        if (event.affectsConfiguration('cp.inlayHints')) {
            emitter.fire();
        }
    });
    const provider: vscode.InlayHintsProvider = {
        onDidChangeInlayHints: emitter.event,
        provideInlayHints(document, range) {
            if (!vscode.workspace.getConfiguration('cp').get<boolean>('inlayHints.enabled', true)) {
                return [];
            }
            return computeInlayHints(document.getText())
                .filter(hint => hint.line >= range.start.line && hint.line <= range.end.line)
                .map(hint => {
                    const item = new vscode.InlayHint(
                        new vscode.Position(hint.line, hint.character),
                        hint.label,
                        vscode.InlayHintKind.Type,
                    );
                    item.tooltip = new vscode.MarkdownString(hint.tooltip);
                    return item;
                });
        },
    };
    const registration = vscode.languages.registerInlayHintsProvider(selector, provider);
    return vscode.Disposable.from(registration, configWatcher, emitter);
}
