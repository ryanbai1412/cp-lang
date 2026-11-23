import { autocompletion, type Completion, type CompletionContext, type CompletionResult, snippet } from '@codemirror/autocomplete';
import type { Extension } from '@codemirror/state';
import { EditorView, hoverTooltip, keymap, tooltips, ViewPlugin } from '@codemirror/view';
import { started } from '@/lib/cpl';
import { LspDoc } from '@/lib/lsp';

type Position = { line: number; character: number };
type Range = { start: Position; end: Position };
type Location = { uri: string; range: Range };
type Markup = string | { kind?: string; value: string };
type Item = {
  label: string;
  kind?: number;
  detail?: string;
  documentation?: Markup;
  sortText?: string;
  insertText?: string;
  insertTextFormat?: number;
  textEdit?: { newText: string };
};

const kinds: Record<number, string> = {
  2: 'method', 3: 'function', 4: 'function', 5: 'property', 6: 'variable', 7: 'class', 8: 'interface',
  9: 'namespace', 10: 'property', 13: 'enum', 14: 'keyword', 20: 'enum', 21: 'constant', 22: 'class', 25: 'type',
};

const text = (m: Markup | Markup[] | undefined) =>
  (Array.isArray(m) ? m : m ? [m] : []).map((x) => (typeof x === 'string' ? x : x.value)).join('\n\n');

// The server's Markdown, as DOM nodes and never as HTML: fenced code, inline
// code, links and paragraphs are all hover and completion docs use.
function markdown(source: string): HTMLElement {
  const root = document.createElement('div');
  root.className = 'cm-lsp-doc';
  const inline = (into: HTMLElement, line: string) => {
    for (const part of line.split(/(`[^`]+`|\[[^\]]+\]\([^)\s]+\))/)) {
      const link = /^\[([^\]]+)\]\(([^)\s]+)\)$/.exec(part);
      if (part.startsWith('`') && part.endsWith('`') && part.length > 1) {
        into.appendChild(document.createElement('code')).textContent = part.slice(1, -1);
      } else if (link) {
        const a = into.appendChild(document.createElement('a'));
        a.href = link[2].replace(/^https:\/\/cp-lang\.org/, '');
        a.target = '_blank';
        a.rel = 'noopener';
        inline(a, link[1]);
      } else if (part) into.appendChild(document.createTextNode(part));
    }
  };
  source.split(/(```\w*\n[\s\S]*?```)/).forEach((block) => {
    const code = /^```\w*\n([\s\S]*?)\n?```$/.exec(block);
    if (code) {
      root.appendChild(document.createElement('pre')).textContent = code[1];
      return;
    }
    for (const para of block.split(/\n\s*\n/)) if (para.trim()) inline(root.appendChild(document.createElement('p')), para.trim());
  });
  return root;
}

type Anchors = Record<string, [number, string][]>;
let anchors: Promise<Anchors> | null = null;

// A std item's docs page, at the anchor of the item declared at or before `line`.
async function stdPage(module: string, line: number) {
  anchors ??= fetch('/std-anchors.json').then((r) => (r.ok ? r.json() : {})) as Promise<Anchors>;
  const items = (await anchors.catch((): Anchors => ({})))[module] ?? [];
  const at = items.filter(([l]) => l <= line).at(-1);
  return `/docs/ref/std/${module}${at ? `#${at[1]}` : ''}`;
}

/**
 * Hover, completion and go to definition (Ctrl/Cmd-click or F12) from the
 * language server, the same answers Neovim, VS Code and JetBrains get.
 * `prefix` is code sent ahead of the editor's text, as the checker gets it.
 */
export function intelligence(prefix: string, editable: boolean): Extension {
  const head = prefix ? `${prefix}\n` : '';
  const above = head.split('\n').length - 1;
  const server = ViewPlugin.fromClass(
    class {
      doc: LspDoc;
      constructor(view: EditorView) {
        this.doc = new LspDoc(() => head + view.state.doc.toString());
      }
      destroy() {
        this.doc.close();
      }
    },
  );
  const toLsp = (view: EditorView, pos: number): Position => {
    const line = view.state.doc.lineAt(pos);
    return { line: line.number - 1 + above, character: pos - line.from };
  };
  const fromLsp = (view: EditorView, p: Position): number | null => {
    const n = p.line - above + 1;
    if (n < 1 || n > view.state.doc.lines) return null;
    const line = view.state.doc.line(n);
    return Math.min(line.from + p.character, line.to);
  };
  const ask = <T>(view: EditorView, method: string, pos: number, extra: object = {}) =>
    view.plugin(server)!.doc.request<T>(method, { position: toLsp(view, pos), ...extra }).catch(() => null);
  // The compiler is a ~1.9 MB download: hovering only loads it once the
  // reader is working in this editor.
  const wanted = (view: EditorView) => started() || view.hasFocus;

  const hover = hoverTooltip(
    async (view, pos) => {
      if (!wanted(view)) return null;
      const h = await ask<{ contents: Markup | Markup[]; range?: Range }>(view, 'textDocument/hover', pos);
      const body = text(h?.contents);
      if (!h || !body.trim()) return null;
      const from = (h.range && fromLsp(view, h.range.start)) ?? pos;
      const to = (h.range && fromLsp(view, h.range.end)) ?? pos;
      return { pos: from, end: to, create: () => ({ dom: markdown(body) }) };
    },
    { hideOnChange: true },
  );

  async function complete(ctx: CompletionContext): Promise<CompletionResult | null> {
    const word = ctx.matchBefore(/\w*/)!;
    const dot = ctx.state.sliceDoc(word.from - 1, word.from) === '.';
    if (!ctx.view || (!ctx.explicit && !dot && word.from === word.to)) return null;
    const context = dot && word.from === word.to ? { triggerKind: 2, triggerCharacter: '.' } : { triggerKind: 1 };
    const res = await ask<{ items: Item[] } | Item[]>(ctx.view, 'textDocument/completion', ctx.pos, { context });
    if (!res || ctx.aborted) return null;
    const options = (Array.isArray(res) ? res : res.items).map((it): Completion => {
      const insert = it.textEdit?.newText ?? it.insertText;
      const doc = text(it.documentation);
      return {
        label: it.label,
        detail: it.detail,
        type: kinds[it.kind ?? 0],
        boost: /^\d/.test(it.sortText ?? '') ? 9 - Number(it.sortText![0]) : 0,
        apply: insert && it.insertTextFormat === 2 ? snippet(insert.replace(/\$(\d+)/g, '${$1}')) : insert,
        info: doc ? () => markdown(doc) : undefined,
      };
    });
    return { from: word.from, options, validFor: /^\w*$/ };
  }

  async function definition(view: EditorView, pos: number) {
    const def = await ask<Location | Location[]>(view, 'textDocument/definition', pos);
    const loc = Array.isArray(def) ? def[0] : def;
    if (!loc) return;
    if (loc.uri === view.plugin(server)!.doc.uri) {
      const from = fromLsp(view, loc.range.start);
      if (from === null) return;
      view.dispatch({ selection: { anchor: from, head: fromLsp(view, loc.range.end) ?? from }, scrollIntoView: true });
      view.focus();
      return;
    }
    const std = /\/std\/(.+)\.cp$/.exec(loc.uri);
    if (std) window.open(await stdPage(std[1].replace(/\//g, '.'), loc.range.start.line + 1), '_blank', 'noopener');
  }

  return [
    // In the page body, so an editor that scrolls its own content can't clip them.
    tooltips({ parent: document.body }),
    server,
    hover,
    editable ? autocompletion({ override: [complete] }) : [],
    keymap.of([{ key: 'F12', run: (view) => (void definition(view, view.state.selection.main.head), true) }]),
    EditorView.domEventHandlers({
      mousedown(event, view) {
        if (event.button !== 0 || !(event.ctrlKey || event.metaKey)) return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (pos === null) return false;
        event.preventDefault();
        void definition(view, pos);
        return true;
      },
    }),
    EditorView.theme({
      '.cm-tooltip': {
        backgroundColor: 'var(--popover)',
        color: 'var(--popover-foreground)',
        border: '1px solid var(--border)',
        borderRadius: '6px',
        fontSize: '12px',
      },
      '.cm-lsp-doc': { maxWidth: 'min(32rem, 90vw)', maxHeight: '18rem', overflow: 'auto', padding: '6px 10px' },
      '.cm-lsp-doc pre': { fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap', margin: '2px 0' },
      '.cm-lsp-doc p': { margin: '6px 0' },
      '.cm-lsp-doc code': { fontFamily: 'var(--font-mono)' },
      '.cm-lsp-doc a': { textDecoration: 'underline' },
      '.cm-tooltip-autocomplete > ul': { fontFamily: 'var(--font-mono)', maxHeight: '14rem' },
      '.cm-tooltip-autocomplete > ul > li[aria-selected]': { backgroundColor: 'var(--accent)', color: 'var(--accent-foreground)' },
      '.cm-completionDetail': { color: 'var(--muted-foreground)', marginLeft: '1em', fontStyle: 'normal' },
    }),
  ];
}
