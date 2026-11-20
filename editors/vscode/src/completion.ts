// Fallback completion for when the server offers none: CP Lang 0.2.2
// reserved words, prelude/stdlib types, prelude functions and decorators.
// Registered only while the server's capabilities have no
// `completionProvider`, so the two never both answer.

import * as vscode from 'vscode';
import {
    CONTEXTUAL_KEYWORDS,
    DECORATORS,
    FILE_DIRECTIVES,
    KEYWORDS,
    PRELUDE_CONSTANTS,
    PRELUDE_TYPES,
    PRELUDE_VALUES,
    STDLIB_TYPES,
    STDLIB_VALUES,
} from './languageData';

function items(names: readonly string[], kind: vscode.CompletionItemKind, detail: string): vscode.CompletionItem[] {
    return names.map(name => {
        const item = new vscode.CompletionItem(name, kind);
        item.detail = detail;
        return item;
    });
}

export function registerFallbackCompletion(selector: vscode.DocumentSelector): vscode.Disposable {
    const words = [
        ...items(KEYWORDS, vscode.CompletionItemKind.Keyword, 'keyword'),
        ...items(CONTEXTUAL_KEYWORDS, vscode.CompletionItemKind.Keyword, 'contextual keyword'),
        ...items(PRELUDE_TYPES, vscode.CompletionItemKind.Class, 'prelude type'),
        ...items(STDLIB_TYPES, vscode.CompletionItemKind.Class, 'stdlib type'),
        ...items(PRELUDE_VALUES, vscode.CompletionItemKind.Function, 'prelude fn'),
        ...items(STDLIB_VALUES, vscode.CompletionItemKind.Function, 'stdlib fn (needs a `use`)'),
        ...items(PRELUDE_CONSTANTS, vscode.CompletionItemKind.Constant, 'prelude constant'),
    ];
    const directives = FILE_DIRECTIVES.map(directive => {
        const item = new vscode.CompletionItem('@!' + directive.name, vscode.CompletionItemKind.Keyword);
        item.detail = directive.detail;
        item.insertText = new vscode.SnippetString(directive.insert);
        item.filterText = directive.name;
        return item;
    });
    const directivesFromAt = FILE_DIRECTIVES.map(directive => {
        const item = new vscode.CompletionItem('@!' + directive.name, vscode.CompletionItemKind.Keyword);
        item.detail = directive.detail;
        item.insertText = new vscode.SnippetString('!' + directive.insert);
        item.filterText = directive.name;
        return item;
    });
    const decorators = DECORATORS.filter(d => d.name !== 'multitest').map(decorator => {
        const item = new vscode.CompletionItem('@' + decorator.name, vscode.CompletionItemKind.Event);
        item.detail = decorator.detail;
        item.insertText = new vscode.SnippetString(decorator.insert);
        item.filterText = decorator.name;
        return item;
    });
    const provider: vscode.CompletionItemProvider = {
        provideCompletionItems(document, position) {
            const before = document.lineAt(position.line).text.slice(0, position.character);
            if (/@![A-Za-z_]*$/.test(before)) {
                return directives;
            }
            if (/@[A-Za-z_]*$/.test(before)) {
                return [...directivesFromAt, ...decorators];
            }
            // Nothing sensible to offer after a dot without types.
            if (/\.[A-Za-z_]*$/.test(before)) {
                return undefined;
            }
            return words;
        },
    };
    return vscode.languages.registerCompletionItemProvider(selector, provider, '@', '!');
}
