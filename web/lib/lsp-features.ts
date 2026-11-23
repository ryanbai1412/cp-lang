// Every capability the language server advertises, and what the browser
// editor does with it (site/R6, handbook/guidelines/site-checklist.md).
// `uses` lists the requests the editor sends; `gap` says why it sends none.
// lib/lsp-parity.test.ts fails when the server advertises a capability that
// is not listed here, or a listed request is no longer sent.
export const lspFeatures: Record<string, { uses: string[] } | { gap: string }> = {
  positionEncoding: { uses: [] },
  textDocumentSync: { uses: [] },
  hoverProvider: { uses: ['textDocument/hover'] },
  completionProvider: { uses: ['textDocument/completion'] },
  definitionProvider: { uses: ['textDocument/definition'] },
  inlayHintProvider: { uses: ['textDocument/inlayHint'] },
  signatureHelpProvider: { uses: ['textDocument/signatureHelp'] },
  documentHighlightProvider: { gap: 'missing: highlight other uses of the name under the cursor' },
  renameProvider: { gap: 'missing: rename a symbol' },
  referencesProvider: { gap: 'missing: find references' },
  codeActionProvider: { gap: 'missing: quick fixes from lints; the editor shows lint diagnostics only' },
  documentFormattingProvider: { gap: 'missing: format the program' },
  documentRangeFormattingProvider: { gap: 'missing: format a selection' },
  documentOnTypeFormattingProvider: { gap: 'missing: format on typing' },
  foldingRangeProvider: { gap: 'missing: folding' },
  semanticTokensProvider: { gap: 'highlighting comes from the shared grammar; semantic tokens not used yet' },
  linkedEditingRangeProvider: { gap: 'missing' },
  typeDefinitionProvider: { gap: 'one program per editor: go to type definition not wired' },
  implementationProvider: { gap: 'one program per editor: go to implementation not wired' },
  callHierarchyProvider: { gap: 'one program per editor: call hierarchy has no panel' },
  documentSymbolProvider: { gap: 'no outline panel in the browser editor' },
  workspaceSymbolProvider: { gap: 'no workspace in the browser' },
  documentLinkProvider: { gap: 'missing' },
  codeLensProvider: { gap: 'missing' },
};
