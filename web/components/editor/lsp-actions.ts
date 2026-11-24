// The language server's editing and navigation requests, beyond the hover,
// completion, hints and definition in intelligence.ts: occurrences,
// references, rename, quick fixes, formatting, folding, outlines, call
// hierarchy, links, lenses, linked edits and semantic tokens. Every one
// asks the server; none of it is worked out here.
import { foldGutter, foldKeymap, foldService } from "@codemirror/language";
import {
  EditorSelection,
  EditorState,
  type Extension,
  type Range as CmRange,
  StateEffect,
  StateField,
  type TransactionSpec,
} from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  keymap,
  showTooltip,
  type Tooltip,
  type ViewUpdate,
  ViewPlugin,
  WidgetType,
} from "@codemirror/view";
import { started } from "@/lib/cpl";
import { capabilities } from "@/lib/lsp";

type Position = { line: number; character: number };
type Range = { start: Position; end: Position };
type Location = { uri: string; range: Range };
type TextEdit = { range: Range; newText: string };
type WorkspaceEdit = {
  changes?: Record<string, TextEdit[]>;
  documentChanges?: { textDocument?: { uri: string }; edits?: TextEdit[] }[];
};
type Symbol = {
  name: string;
  detail?: string;
  range?: Range;
  selectionRange?: Range;
  location?: Location;
  children?: Symbol[];
};
type CallItem = { name: string; uri: string; range: Range; selectionRange: Range };

export type Lsp = {
  editable: boolean;
  uri: (view: EditorView) => string;
  request: <T>(view: EditorView, method: string, params?: object) => Promise<T | null>;
  toLsp: (view: EditorView, pos: number) => Position;
  fromLsp: (view: EditorView, p: Position) => number | null;
  wanted: (view: EditorView) => boolean;
  go: (view: EditorView, loc: Location) => void | Promise<void>;
};

const FORMAT = { tabSize: 4, insertSpaces: true };

// A small menu of choices at a position: quick fixes, symbols, callers.
type Choice = { label: string; detail?: string; run: () => void };
const setMenu = StateEffect.define<Tooltip | null>();
const menuField = StateField.define<Tooltip | null>({
  create: () => null,
  update: (tip, tr) => {
    for (const e of tr.effects) if (e.is(setMenu)) return e.value;
    return tr.docChanged ? null : tip;
  },
  provide: (f) => showTooltip.from(f),
});

function menu(view: EditorView, pos: number, title: string, choices: Choice[]) {
  const close = () => {
    view.dispatch({ effects: setMenu.of(null) });
    view.focus();
  };
  const create = () => {
    const dom = document.createElement("div");
    dom.className = "cm-lsp-menu";
    dom.setAttribute("role", "menu");
    const head = document.createElement("div");
    head.className = "cm-lsp-menu-title";
    head.textContent = choices.length ? title : `${title}: none`;
    dom.append(head);
    const buttons = choices.map((c) => {
      const b = document.createElement("button");
      b.type = "button";
      b.setAttribute("role", "menuitem");
      b.textContent = c.label;
      if (c.detail) {
        const d = document.createElement("span");
        d.className = "cm-completionDetail";
        d.textContent = c.detail;
        b.append(d);
      }
      b.onclick = () => {
        close();
        c.run();
      };
      dom.append(b);
      return b;
    });
    dom.onkeydown = (e) => {
      const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
      if (e.key === "Escape") close();
      else if (e.key === "ArrowDown") buttons[(at + 1) % buttons.length]?.focus();
      else if (e.key === "ArrowUp") buttons[(at - 1 + buttons.length) % buttons.length]?.focus();
      else return;
      e.preventDefault();
    };
    return { dom, mount: () => (buttons[0] ?? dom).focus() };
  };
  view.dispatch({ effects: setMenu.of({ pos, above: false, create }) });
}

function editsFor(uri: string, edit: WorkspaceEdit | null): TextEdit[] {
  if (!edit) return [];
  if (edit.changes) return edit.changes[uri] ?? [];
  return (edit.documentChanges ?? []).filter((d) => d.textDocument?.uri === uri).flatMap((d) => d.edits ?? []);
}

function apply(lsp: Lsp, view: EditorView, edits: TextEdit[], extra: TransactionSpec = {}) {
  const changes = edits.flatMap((e) => {
    const from = lsp.fromLsp(view, e.range.start);
    const to = lsp.fromLsp(view, e.range.end);
    return from === null || to === null ? [] : [{ from, to, insert: e.newText }];
  });
  if (changes.length) view.dispatch({ changes, ...extra });
}

const flat = (symbols: Symbol[], depth = 0): [Symbol, number][] =>
  symbols.flatMap((s) => [[s, depth] as [Symbol, number], ...flat(s.children ?? [], depth + 1)]);

/** Fetches something for the whole document after edits settle; `show` gets it. */
function onDocument<T>(lsp: Lsp, ms: number, fetch: (view: EditorView) => Promise<T | null>, show: (view: EditorView, got: T) => void) {
  return ViewPlugin.fromClass(
    class {
      timer = 0;
      asked = 0;
      constructor(readonly view: EditorView) {
        this.schedule(0);
      }
      update(u: ViewUpdate) {
        if (u.docChanged || u.focusChanged) this.schedule(ms);
      }
      schedule(wait: number) {
        clearTimeout(this.timer);
        this.timer = window.setTimeout(() => void this.refresh(), wait);
      }
      async refresh() {
        if (!lsp.wanted(this.view)) return;
        const ask = ++this.asked;
        const doc = this.view.state.doc;
        const got = await fetch(this.view);
        if (got !== null && ask === this.asked && doc === this.view.state.doc) show(this.view, got);
      }
      destroy() {
        clearTimeout(this.timer);
      }
    },
  );
}

/** A decoration set a plugin replaces, mapped through edits until then. */
function decorations() {
  const set = StateEffect.define<DecorationSet>();
  const field = StateField.define<DecorationSet>({
    create: () => Decoration.none,
    update: (deco, tr) => {
      for (const e of tr.effects) if (e.is(set)) return e.value;
      return deco.map(tr.changes);
    },
    provide: (f) => EditorView.decorations.from(f),
  });
  return { set, field };
}

class LensWidget extends WidgetType {
  constructor(readonly text: string) {
    super();
  }
  eq(other: LensWidget) {
    return other.text === this.text;
  }
  toDOM() {
    const el = document.createElement("div");
    el.className = "cm-lsp-lens";
    el.textContent = this.text;
    return el;
  }
}

/** The extensions, and `link` for a Ctrl+click to try before going to a definition. */
export function lspActions(lsp: Lsp): { extension: Extension; link: (view: EditorView, pos: number) => Promise<boolean> } {
  const at = (view: EditorView, pos = view.state.selection.main.head) => ({ position: lsp.toLsp(view, pos) });
  const span = (view: EditorView, r: Range) => {
    const from = lsp.fromLsp(view, r.start);
    const to = lsp.fromLsp(view, r.end);
    return from === null || to === null ? null : { from, to };
  };

  // The other uses of the name under the cursor, from `textDocument/documentHighlight`.
  const occurrences = decorations();
  const occurrence = Decoration.mark({ class: "cm-lsp-occurrence" });
  const written = Decoration.mark({ class: "cm-lsp-occurrence cm-lsp-written" });
  const highlights = ViewPlugin.fromClass(
    class {
      timer = 0;
      asked = 0;
      constructor(readonly view: EditorView) {}
      update(u: ViewUpdate) {
        if (!u.selectionSet && !u.docChanged) return;
        clearTimeout(this.timer);
        this.timer = window.setTimeout(() => void this.refresh(), 200);
      }
      async refresh() {
        const view = this.view;
        if (!lsp.wanted(view)) return;
        const ask = ++this.asked;
        const pos = view.state.selection.main.head;
        const got = await lsp.request<{ range: Range; kind?: number }[]>(view, "textDocument/documentHighlight", at(view, pos));
        if (ask !== this.asked || pos !== view.state.selection.main.head) return;
        const marks = (got ?? []).flatMap((h) => {
          const s = span(view, h.range);
          return s && s.from < s.to ? [(h.kind === 3 ? written : occurrence).range(s.from, s.to)] : [];
        });
        view.dispatch({ effects: occurrences.set.of(Decoration.set(marks, true)) });
      }
      destroy() {
        clearTimeout(this.timer);
      }
    },
  );

  // Every reference to the name, selected at once (Shift+F12).
  async function references(view: EditorView) {
    const got = await lsp.request<Location[]>(view, "textDocument/references", { ...at(view), context: { includeDeclaration: true } });
    const here = (got ?? []).filter((l) => l.uri === lsp.uri(view)).flatMap((l) => span(view, l.range) ?? []);
    if (!here.length) return;
    const main = view.state.selection.main.head;
    view.dispatch({
      selection: EditorSelection.create(
        here.map((s) => EditorSelection.range(s.from, s.to)),
        Math.max(0, here.findIndex((s) => s.from <= main && main <= s.to)),
      ),
      scrollIntoView: true,
    });
  }

  async function jump(view: EditorView, method: string, title: string) {
    const got = await lsp.request<Location | Location[]>(view, method, at(view));
    const all = (Array.isArray(got) ? got : got ? [got] : []);
    if (all.length === 1) return void lsp.go(view, all[0]);
    menu(view, view.state.selection.main.head, title, all.map((l) => ({
      label: l.uri === lsp.uri(view) ? `line ${l.range.start.line + 1}` : l.uri.replace(/^.*\//, ""),
      run: () => void lsp.go(view, l),
    })));
  }

  // F2: rename the name everywhere the server says it is used.
  async function rename(view: EditorView) {
    const prep = await lsp.request<Range | { range: Range; placeholder: string } | { defaultBehavior: boolean }>(view, "textDocument/prepareRename", at(view));
    if (!prep) return;
    const range = "start" in prep ? prep : "range" in prep ? prep.range : null;
    const s = range && span(view, range);
    const placeholder = prep && "placeholder" in prep ? prep.placeholder : s ? view.state.sliceDoc(s.from, s.to) : "";
    const newName = window.prompt("Rename to", placeholder);
    if (!newName || newName === placeholder) return;
    const edit = await lsp.request<WorkspaceEdit>(view, "textDocument/rename", { ...at(view), newName });
    apply(lsp, view, editsFor(lsp.uri(view), edit), { userEvent: "rename" });
  }

  // Ctrl+.: the quick fixes and refactorings for the selection.
  async function codeActions(view: EditorView) {
    const sel = view.state.selection.main;
    const got = await lsp.request<{ title: string; kind?: string; edit?: WorkspaceEdit }[]>(view, "textDocument/codeAction", {
      range: { start: lsp.toLsp(view, sel.from), end: lsp.toLsp(view, sel.to) },
      context: { diagnostics: [] },
    });
    const actions = (got ?? []).filter((a) => editsFor(lsp.uri(view), a.edit ?? null).length);
    menu(view, sel.head, "Quick fixes", actions.map((a) => ({
      label: a.title,
      detail: a.kind,
      run: () => apply(lsp, view, editsFor(lsp.uri(view), a.edit!), { userEvent: "codeAction" }),
    })));
  }

  // Shift+Alt+F: `cpl fmt`, the selection or the whole file.
  async function format(view: EditorView) {
    const sel = view.state.selection.main;
    const edits = sel.empty
      ? await lsp.request<TextEdit[]>(view, "textDocument/formatting", { options: FORMAT })
      : await lsp.request<TextEdit[]>(view, "textDocument/rangeFormatting", {
          range: { start: lsp.toLsp(view, sel.from), end: lsp.toLsp(view, sel.to) },
          options: FORMAT,
        });
    apply(lsp, view, edits ?? [], { userEvent: "format" });
  }

  // Formatting as you type the characters the server names.
  // The trigger characters are asked for once the compiler runs or the reader
  // types, so an editor that is only shown doesn't fetch cpl.wasm.
  const onType = ViewPlugin.fromClass(
    class {
      triggers: Promise<string[]> | null = null;
      constructor(readonly view: EditorView) {
        if (started()) void this.load();
      }
      load() {
        return (this.triggers ??= capabilities().then((caps) => {
          const c = caps.documentOnTypeFormattingProvider as { firstTriggerCharacter?: string; moreTriggerCharacter?: string[] } | undefined;
          return c ? [c.firstTriggerCharacter ?? "", ...(c.moreTriggerCharacter ?? [])].filter(Boolean) : [];
        }).catch(() => []));
      }
      update(u: ViewUpdate) {
        if (!u.docChanged || !u.transactions.some((tr) => tr.isUserEvent("input.type"))) return;
        const { changes, state: { doc } } = u;
        void this.load().then((triggers) => {
          let typed: { ch: string; pos: number } | null = null;
          changes.iterChanges((_fa, _ta, _fb, toB, text) => {
            const s = text.toString();
            const ch = triggers.find((t) => s.endsWith(t));
            if (ch) typed = { ch, pos: toB };
          });
          if (!typed || this.view.state.doc !== doc) return;
          const { ch, pos } = typed;
          void lsp.request<TextEdit[]>(this.view, "textDocument/onTypeFormatting", { ...at(this.view, pos), ch, options: FORMAT }).then((edits) => {
            if (edits && this.view.state.doc === doc) apply(lsp, this.view, edits, { userEvent: "format" });
          });
        });
      }
    },
  );

  // Editing a name edits its other uses with it (linkedEditingRange).
  const setLinked = StateEffect.define<{ from: number; to: number }[]>();
  const linkedField = StateField.define<{ from: number; to: number }[]>({
    create: () => [],
    update: (ranges, tr) => {
      for (const e of tr.effects) if (e.is(setLinked)) return e.value;
      return ranges.map((r) => ({ from: tr.changes.mapPos(r.from, -1), to: tr.changes.mapPos(r.to, 1) }));
    },
  });
  const linkedFilter = EditorState.transactionFilter.of((tr) => {
    const ranges = tr.startState.field(linkedField, false);
    if (!ranges?.length || !tr.docChanged || !(tr.isUserEvent("input") || tr.isUserEvent("delete"))) return tr;
    const mirrored: { from: number; to: number; insert: string }[] = [];
    let inside = true;
    tr.changes.iterChanges((fromA, toA, _fb, _tb, text) => {
      const home = ranges.find((r) => r.from <= fromA && toA <= r.to);
      if (!home) return void (inside = false);
      for (const r of ranges) {
        if (r === home) continue;
        const from = r.from + (fromA - home.from);
        mirrored.push({ from, to: from + (toA - fromA), insert: text.toString() });
      }
    });
    if (!inside || !mirrored.length || tr.changes.desc.length !== tr.startState.doc.length) return tr;
    return [tr, { changes: mirrored }];
  });
  const linked = ViewPlugin.fromClass(
    class {
      timer = 0;
      constructor(readonly view: EditorView) {}
      update(u: ViewUpdate) {
        if (!u.selectionSet || u.docChanged) return;
        clearTimeout(this.timer);
        this.timer = window.setTimeout(() => void this.refresh(), 200);
      }
      async refresh() {
        const view = this.view;
        if (!lsp.wanted(view)) return;
        const pos = view.state.selection.main.head;
        const doc = view.state.doc;
        const got = await lsp.request<{ ranges: Range[] }>(view, "textDocument/linkedEditingRange", at(view, pos));
        if (doc !== view.state.doc) return;
        const ranges = (got?.ranges ?? []).flatMap((r) => span(view, r) ?? []);
        view.dispatch({ effects: setLinked.of(ranges.some((r) => r.from <= pos && pos <= r.to) ? ranges : []) });
      }
      destroy() {
        clearTimeout(this.timer);
      }
    },
  );

  // Folding where the server says a block is.
  const setFolds = StateEffect.define<Map<number, number>>();
  const foldsField = StateField.define<Map<number, number>>({
    create: () => new Map(),
    update: (folds, tr) => {
      for (const e of tr.effects) if (e.is(setFolds)) return e.value;
      return tr.docChanged ? new Map() : folds;
    },
  });
  const folds = onDocument(
    lsp,
    500,
    (view) => lsp.request<{ startLine: number; endLine: number }[]>(view, "textDocument/foldingRange"),
    (view, got) => {
      const map = new Map<number, number>();
      for (const r of got) {
        const start = lsp.fromLsp(view, { line: r.startLine, character: 0 });
        const end = lsp.fromLsp(view, { line: r.endLine, character: 0 });
        if (start === null || end === null) continue;
        const from = view.state.doc.lineAt(start);
        const to = view.state.doc.lineAt(end);
        if (to.number > from.number) map.set(from.from, to.to);
      }
      view.dispatch({ effects: setFolds.of(map) });
    },
  );
  const fold = foldService.of((state, lineStart, lineEnd) => {
    const end = state.field(foldsField, false)?.get(lineStart);
    return end === undefined ? null : { from: lineEnd, to: end };
  });

  // Ctrl+Shift+O: the file's outline; Ctrl+T: symbols by the word under the cursor.
  async function outline(view: EditorView) {
    const got = await lsp.request<Symbol[]>(view, "textDocument/documentSymbol");
    menu(view, view.state.selection.main.head, "Symbols", flat(got ?? []).flatMap(([s, depth]) => {
      const r = s.selectionRange ?? s.range ?? s.location?.range;
      return r ? [{ label: `${"  ".repeat(depth)}${s.name}`, detail: s.detail, run: () => void lsp.go(view, { uri: lsp.uri(view), range: r }) }] : [];
    }));
  }
  async function symbols(view: EditorView) {
    const word = view.state.wordAt(view.state.selection.main.head);
    const query = word ? view.state.sliceDoc(word.from, word.to) : "";
    const got = await lsp.request<Symbol[]>(view, "workspace/symbol", { query });
    menu(view, view.state.selection.main.head, `Symbols matching "${query}"`, (got ?? []).flatMap((s) =>
      s.location ? [{ label: s.name, detail: s.location.uri.replace(/^.*\//, ""), run: () => void lsp.go(view, s.location!) }] : []));
  }

  // Shift+Alt+H: who calls the fn under the cursor.
  async function callers(view: EditorView) {
    const items = await lsp.request<CallItem[]>(view, "textDocument/prepareCallHierarchy", at(view));
    const item = items?.[0];
    if (!item) return;
    const calls = await lsp.request<{ from: CallItem; fromRanges: Range[] }[]>(view, "callHierarchy/incomingCalls", { item });
    menu(view, view.state.selection.main.head, `Calls to ${item.name}`, (calls ?? []).map((c) => ({
      label: c.from.name,
      detail: c.from.uri === lsp.uri(view) ? `line ${(c.fromRanges[0] ?? c.from.range).start.line + 1}` : c.from.uri.replace(/^.*\//, ""),
      run: () => void lsp.go(view, { uri: c.from.uri, range: c.fromRanges[0] ?? c.from.selectionRange }),
    })));
  }

  // Ctrl+click on a `use` opens what it names (documentLink).
  async function link(view: EditorView, pos: number): Promise<boolean> {
    const got = await lsp.request<{ range: Range; target?: string }[]>(view, "textDocument/documentLink");
    const hit = (got ?? []).find((l) => {
      const s = span(view, l.range);
      return l.target && s && s.from <= pos && pos <= s.to;
    });
    if (!hit) return false;
    await lsp.go(view, { uri: hit.target!, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } } });
    return true;
  }

  // What the server says about the file as a whole, above its first line (codeLens).
  const lenses = decorations();
  const lensPlugin = onDocument(
    lsp,
    800,
    (view) => lsp.request<{ range: Range; command?: { title: string } }[]>(view, "textDocument/codeLens"),
    (view, got) => {
      const marks: CmRange<Decoration>[] = got.flatMap((l) => {
        const pos = lsp.fromLsp(view, l.range.start) ?? 0;
        return l.command?.title ? [Decoration.widget({ widget: new LensWidget(l.command.title), block: true, side: -1 }).range(view.state.doc.lineAt(pos).from)] : [];
      });
      view.dispatch({ effects: lenses.set.of(Decoration.set(marks, true)) });
    },
  );

  // What each name is, which the grammar can't know (semanticTokens).
  const tokens = decorations();
  const tokenPlugin = onDocument(
    lsp,
    300,
    async (view) => {
      const caps = await capabilities();
      const legend = (caps.semanticTokensProvider as { legend?: { tokenTypes: string[]; tokenModifiers: string[] } } | undefined)?.legend;
      const got = legend ? await lsp.request<{ data: number[] }>(view, "textDocument/semanticTokens/full") : null;
      return got && legend ? { data: got.data, legend } : null;
    },
    (view, { data, legend }) => {
      const marks: CmRange<Decoration>[] = [];
      let line = 0;
      let character = 0;
      for (let i = 0; i + 4 < data.length; i += 5) {
        line += data[i];
        character = data[i] ? data[i + 1] : character + data[i + 1];
        const from = lsp.fromLsp(view, { line, character });
        if (from === null) continue;
        const to = Math.min(from + data[i + 2], view.state.doc.lineAt(from).to);
        const mods = legend.tokenModifiers.filter((_, bit) => data[i + 4] & (1 << bit));
        const cls = [`cm-sem-${legend.tokenTypes[data[i + 3]]}`, ...mods.map((m) => `cm-sem-${m}`)].join(" ");
        if (to > from) marks.push(Decoration.mark({ class: cls }).range(from, to));
      }
      view.dispatch({ effects: tokens.set.of(Decoration.set(marks, true)) });
    },
  );

  const run = (f: (view: EditorView) => Promise<unknown>) => (view: EditorView) => (void f(view), true);
  const extension = [
    menuField,
    occurrences.field,
    highlights,
    folds,
    foldsField,
    fold,
    lsp.editable ? foldGutter() : [],
    lenses.field,
    lensPlugin,
    tokens.field,
    tokenPlugin,
    lsp.editable ? [onType, linkedField, linkedFilter, linked, EditorState.allowMultipleSelections.of(true)] : [],
    keymap.of([
      ...foldKeymap,
      { key: "Shift-F12", run: run(references) },
      { key: "Mod-F12", run: run((v) => jump(v, "textDocument/implementation", "Implementations")) },
      { key: "Mod-Shift-F12", run: run((v) => jump(v, "textDocument/typeDefinition", "Type definition")) },
      { key: "Mod-Shift-o", run: run(outline) },
      { key: "Mod-t", run: run(symbols) },
      { key: "Shift-Alt-h", run: run(callers) },
      ...(lsp.editable
        ? [
            { key: "F2", run: run(rename) },
            { key: "Mod-.", run: run(codeActions) },
            { key: "Shift-Alt-f", run: run(format) },
          ]
        : []),
    ]),
    EditorView.theme({
      ".cm-lsp-occurrence": { backgroundColor: "color-mix(in srgb, var(--accent) 60%, transparent)", borderRadius: "2px" },
      ".cm-lsp-written": { outline: "1px solid var(--border)" },
      ".cm-lsp-lens": { color: "var(--muted-foreground)", fontSize: "0.8em", padding: "0 0 2px" },
      ".cm-lsp-menu": { display: "flex", flexDirection: "column", padding: "4px", maxHeight: "16rem", overflow: "auto", fontFamily: "var(--font-mono)" },
      ".cm-lsp-menu-title": { color: "var(--muted-foreground)", padding: "2px 6px" },
      ".cm-lsp-menu button": { textAlign: "left", padding: "2px 6px", borderRadius: "4px", whiteSpace: "pre" },
      ".cm-lsp-menu button:hover, .cm-lsp-menu button:focus": { backgroundColor: "var(--accent)", color: "var(--accent-foreground)", outline: "none" },
      ".cm-sem-parameter": { fontStyle: "italic" },
      ".cm-sem-readonly": { fontWeight: "500" },
    }),
  ];
  return { extension, link };
}
