import {
  acceptCompletion,
  autocompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
  completionStatus,
  snippet,
  startCompletion,
} from "@codemirror/autocomplete";
import {
  type Extension,
  Prec,
  StateEffect,
  StateField,
} from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  hoverTooltip,
  keymap,
  showTooltip,
  type Tooltip,
  tooltips,
  type ViewUpdate,
  ViewPlugin,
  WidgetType,
} from "@codemirror/view";
import { compilerState, cpl, started } from "@/lib/cpl";
import { LspDoc } from "@/lib/lsp";
import { lspActions } from "./lsp-actions";

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
  /** An item from a std module not given yet: its docs come from resolve. */
  data?: { module?: string };
};

const kinds: Record<number, string> = {
  2: "method",
  3: "function",
  4: "function",
  5: "property",
  6: "variable",
  7: "class",
  8: "interface",
  9: "namespace",
  10: "property",
  13: "enum",
  14: "keyword",
  20: "enum",
  21: "constant",
  22: "class",
  25: "type",
};

const text = (m: Markup | Markup[] | undefined) =>
  (Array.isArray(m) ? m : m ? [m] : [])
    .map((x) => (typeof x === "string" ? x : x.value))
    .join("\n\n");

// The server's Markdown, as DOM nodes and never as HTML: fenced code, inline
// code, links and paragraphs are all hover and completion docs use.
function markdown(source: string): HTMLElement {
  const root = document.createElement("div");
  root.className = "cm-lsp-doc";
  const inline = (into: HTMLElement, line: string) => {
    for (const part of line.split(/(`[^`]+`|\[[^\]]+\]\([^)\s]+\))/)) {
      const link = /^\[([^\]]+)\]\(([^)\s]+)\)$/.exec(part);
      if (part.startsWith("`") && part.endsWith("`") && part.length > 1) {
        into.appendChild(document.createElement("code")).textContent =
          part.slice(1, -1);
      } else if (link) {
        const a = into.appendChild(document.createElement("a"));
        a.href = link[2].replace(/^https:\/\/cp-lang\.org/, "");
        a.target = "_blank";
        a.rel = "noopener";
        inline(a, link[1]);
      } else if (part) into.appendChild(document.createTextNode(part));
    }
  };
  source.split(/(```\w*\n[\s\S]*?```)/).forEach((block) => {
    const code = /^```\w*\n([\s\S]*?)\n?```$/.exec(block);
    if (code) {
      root.appendChild(document.createElement("pre")).textContent = code[1];
      return;
    }
    for (const para of block.split(/\n\s*\n/))
      if (para.trim())
        inline(root.appendChild(document.createElement("p")), para.trim());
  });
  return root;
}

type StdItem = [line: number, anchor: string, doc: string];
const modules = new Map<string, Promise<StdItem[]>>();

// The std item declared at or before `line` of `module`, from the docs build.
async function stdItem(module: string, line: number) {
  if (!modules.has(module)) {
    const items = fetch(`/std-docs/${module}.json`).then((r) =>
      r.ok ? r.json() : [],
    ) as Promise<StdItem[]>;
    modules.set(
      module,
      items.catch(() => []),
    );
  }
  return (await modules.get(module)!).filter(([l]) => l <= line).at(-1);
}

const stdTarget = (loc: Location | null) => {
  const std = loc && /\/std\/(.+)\.cp$/.exec(loc.uri);
  return std
    ? { module: std[1].replace(/\//g, "."), line: loc!.range.start.line + 1 }
    : null;
};

type InlayHint = {
  position: Position;
  label: string | { value: string }[];
  paddingLeft?: boolean;
  paddingRight?: boolean;
};

const hintsKey = "cpl.inlayHints";
const hintsEvent = "cpl-inlay-hints";

/** Whether editors show inlay type hints; on unless the reader turned them off. */
export const inlayHintsOn = () =>
  typeof localStorage === "undefined" || localStorage.getItem(hintsKey) !== "off";

/** Turns inlay hints on or off in every editor on the page, and remembers it. */
export function setInlayHints(on: boolean) {
  localStorage.setItem(hintsKey, on ? "on" : "off");
  window.dispatchEvent(new Event(hintsEvent));
}

type SignatureHelp = {
  signatures: {
    label: string;
    parameters?: { label: string | [number, number] }[];
  }[];
  activeSignature?: number;
  activeParameter?: number;
};

const setSignature = StateEffect.define<Tooltip | null>();
const signatureField = StateField.define<Tooltip | null>({
  create: () => null,
  update(tip, tr) {
    for (const e of tr.effects) if (e.is(setSignature)) return e.value;
    return tip && tr.docChanged ? { ...tip, pos: tr.changes.mapPos(tip.pos) } : tip;
  },
  provide: (f) => showTooltip.from(f),
});

// The signature with the parameter being typed in bold.
function signatureDom(help: SignatureHelp) {
  const sig = help.signatures[help.activeSignature ?? 0];
  const dom = document.createElement("div");
  dom.className = "cm-lsp-doc cm-signature-help";
  const param = sig.parameters?.[help.activeParameter ?? 0]?.label;
  const [from, to] =
    typeof param === "string"
      ? [sig.label.indexOf(param), sig.label.indexOf(param) + param.length]
      : (param ?? [-1, -1]);
  if (from < 0) dom.textContent = sig.label;
  else {
    const b = document.createElement("b");
    b.textContent = sig.label.slice(from, to);
    dom.append(sig.label.slice(0, from), b, sig.label.slice(to));
  }
  return dom;
}

class HintWidget extends WidgetType {
  constructor(readonly label: string) {
    super();
  }
  eq(other: HintWidget) {
    return other.label === this.label;
  }
  toDOM() {
    const span = document.createElement("span");
    span.className = "cm-inlay-hint";
    span.textContent = this.label;
    return span;
  }
  ignoreEvent() {
    return true;
  }
}

const setHints = StateEffect.define<DecorationSet>();
const hintField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(hints, tr) {
    hints = hints.map(tr.changes);
    for (const e of tr.effects) if (e.is(setHints)) hints = e.value;
    return hints;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/**
 * Hover, completion and go to definition (Ctrl/Cmd-click or F12) from the
 * language server, the same answers Neovim, VS Code and JetBrains get.
 * `prefix` is code sent ahead of the editor's text, as the checker gets it.
 */
export function intelligence(prefix: string, editable: boolean): Extension {
  const head = prefix ? `${prefix}\n` : "";
  const above = head.split("\n").length - 1;
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
  // A std module's docs ship with its text, fetched the first time an item
  // from it shows its info.
  const resolveDocs = async (view: EditorView, item: Item) => {
    await cpl.provideStd(item.data!.module!).catch(() => {});
    const full = await LspDoc.request<Item>("completionItem/resolve", item)
      .catch(() => null);
    const doc = text(full?.documentation);
    return doc ? markdown(doc) : null;
  };
  const ask = <T>(
    view: EditorView,
    method: string,
    pos: number,
    extra: object = {},
  ) =>
    view
      .plugin(server)!
      .doc.request<T>(method, { position: toLsp(view, pos), ...extra })
      .catch(() => null);
  // The compiler is a ~1.9 MB download: hovering only loads it once the
  // reader is working in this editor.
  const wanted = (view: EditorView) => started() || view.hasFocus;

  const hover = hoverTooltip(
    async (view, pos) => {
      if (!wanted(view)) return null;
      const [h, def] = await Promise.all([
        ask<{ contents: Markup | Markup[]; range?: Range }>(
          view,
          "textDocument/hover",
          pos,
        ),
        ask<Location | Location[]>(view, "textDocument/definition", pos),
      ]);
      const std = stdTarget(Array.isArray(def) ? def[0] : def);
      const item = std && (await stdItem(std.module, std.line));
      const page =
        std && `/docs/ref/std/${std.module}${item ? `#${item[1]}` : ""}`;
      const body = [text(h?.contents), item?.[2], page && `[Docs](${page})`]
        .filter(Boolean)
        .join("\n\n");
      if (!h || !body.trim()) return null;
      const from = (h.range && fromLsp(view, h.range.start)) ?? pos;
      const to = (h.range && fromLsp(view, h.range.end)) ?? pos;
      return { pos: from, end: to, create: () => ({ dom: markdown(body) }) };
    },
    { hideOnChange: true },
  );

  // Inlay hints: the type of each binding written without one, and the
  // parameter a literal argument fills, from `textDocument/inlayHint`.
  const hints = ViewPlugin.fromClass(
    class {
      timer = 0;
      asked = 0;
      toggle = () => this.schedule(0);
      constructor(readonly view: EditorView) {
        window.addEventListener(hintsEvent, this.toggle);
        this.schedule(0);
      }
      update(u: ViewUpdate) {
        if (u.docChanged || u.focusChanged) this.schedule(300);
      }
      schedule(ms: number) {
        clearTimeout(this.timer);
        this.timer = window.setTimeout(() => void this.refresh(), ms);
      }
      async refresh() {
        const view = this.view;
        const ask = ++this.asked;
        const show = (set: DecorationSet) =>
          ask === this.asked && view.dispatch({ effects: setHints.of(set) });
        if (!inlayHintsOn()) return show(Decoration.none);
        if (!wanted(view)) return;
        const end = view.state.doc.length;
        const res = await view
          .plugin(server)!
          .doc.request<InlayHint[]>("textDocument/inlayHint", {
            range: { start: toLsp(view, 0), end: toLsp(view, end) },
          })
          .catch(() => null);
        if (!res || !inlayHintsOn()) return;
        const marks = res.flatMap((h) => {
          const at = fromLsp(view, h.position);
          if (at === null) return [];
          const label =
            typeof h.label === "string"
              ? h.label
              : h.label.map((p) => p.value).join("");
          const text = `${h.paddingLeft ? " " : ""}${label}${h.paddingRight ? " " : ""}`;
          return [
            Decoration.widget({ widget: new HintWidget(text), side: 1 }).range(at),
          ];
        });
        show(Decoration.set(marks, true));
      }
      destroy() {
        clearTimeout(this.timer);
        window.removeEventListener(hintsEvent, this.toggle);
      }
    },
  );

  let reopened = false;
  const note = (from: number, label: string): CompletionResult => ({
    from,
    filter: false,
    options: [{ label, type: "text", apply: () => {} }],
  });

  async function complete(
    ctx: CompletionContext,
  ): Promise<CompletionResult | null> {
    const word = ctx.matchBefore(/\w*/)!;
    const dot = ctx.state.sliceDoc(word.from - 1, word.from) === ".";
    // `(` and `,` open the call's argument names, `{` a `use`'s items.
    const opener =
      word.from === word.to && !ctx.explicit
        ? ctx.matchBefore(/[(,{] ?$/)
        : null;
    if (!ctx.view || (!ctx.explicit && !dot && !opener && word.from === word.to))
      return null;
    const context =
      dot && word.from === word.to
        ? { triggerKind: 2, triggerCharacter: "." }
        : opener
          ? { triggerKind: 2, triggerCharacter: opener.text[0] }
          : { triggerKind: 1 };
    // Until cpl.wasm answers, the menu says it is loading or didn't load, and
    // opens again with the real items once the request settles (complete/U30).
    const view = ctx.view;
    const state = compilerState();
    if (state === "failed" && reopened) {
      reopened = false;
      return note(word.from, "The compiler didn't load, so there are no suggestions. Press Ctrl+Space to try again.");
    }
    reopened = false;
    const pending = ask<{ items: Item[] } | Item[]>(
      view,
      "textDocument/completion",
      ctx.pos,
      { context },
    );
    if (state !== "ready") {
      void pending.then(() => {
        if (completionStatus(view.state) !== "active") return;
        reopened = true;
        startCompletion(view);
      });
      return note(word.from, "Loading the compiler for suggestions…");
    }
    const res = await pending;
    if (!res || ctx.aborted) return null;
    const options = (Array.isArray(res) ? res : res.items).map(
      (it): Completion => {
        const insert = it.textEdit?.newText ?? it.insertText;
        const doc = text(it.documentation);
        return {
          label: it.label,
          detail: it.detail,
          type: kinds[it.kind ?? 0],
          boost: /^\d/.test(it.sortText ?? "")
            ? 9 - Number(it.sortText![0])
            : 0,
          apply:
            insert && it.insertTextFormat === 2
              ? snippet(insert.replace(/\$(\d+)/g, "${$1}"))
              : insert,
          info: doc
            ? () => markdown(doc)
            : it.data?.module
              ? () => resolveDocs(view, it)
              : undefined,
        };
      },
    );
    // After `(`, `,` or `{` the server sends only argument names or `use` items,
    // so the next word asks again rather than filtering those.
    return { from: word.from, options, validFor: opener ? undefined : /^\w*$/ };
  }

  async function definition(view: EditorView, pos: number) {
    const def = await ask<Location | Location[]>(
      view,
      "textDocument/definition",
      pos,
    );
    const loc = Array.isArray(def) ? def[0] : def;
    if (loc) await go(view, loc);
  }

  // Shows `loc`: a place in this editor, or the docs page of a std item.
  async function go(view: EditorView, loc: Location) {
    if (loc.uri === view.plugin(server)!.doc.uri) {
      const from = fromLsp(view, loc.range.start);
      if (from === null) return;
      view.dispatch({
        selection: { anchor: from, head: fromLsp(view, loc.range.end) ?? from },
        scrollIntoView: true,
      });
      view.focus();
      return;
    }
    const std = stdTarget(loc);
    if (!std) return;
    const item = await stdItem(std.module, std.line);
    window.open(
      `/docs/ref/std/${std.module}${item ? `#${item[1]}` : ""}`,
      "_blank",
      "noopener",
    );
  }

  const actions = lspActions({
    editable,
    uri: (view) => view.plugin(server)!.doc.uri,
    request: <T,>(view: EditorView, method: string, params: object = {}) =>
      view.plugin(server)!.doc.request<T>(method, params).catch(() => null),
    toLsp,
    fromLsp,
    wanted,
    go,
  });

  // Parameter hints while typing a call, from `textDocument/signatureHelp`.
  const signature = ViewPlugin.fromClass(
    class {
      timer = 0;
      asked = 0;
      constructor(readonly view: EditorView) {}
      update(u: ViewUpdate) {
        if (!(u.docChanged || u.selectionSet) || !u.view.hasFocus) return;
        clearTimeout(this.timer);
        this.timer = window.setTimeout(() => void this.refresh(), 150);
      }
      async refresh() {
        const view = this.view;
        const ask = ++this.asked;
        const pos = view.state.selection.main.head;
        const help = await view
          .plugin(server)!
          .doc.request<SignatureHelp>("textDocument/signatureHelp", {
            position: toLsp(view, pos),
          })
          .catch(() => null);
        if (ask !== this.asked || pos !== view.state.selection.main.head) return;
        const tip: Tooltip | null = help?.signatures?.length
          ? { pos, above: true, create: () => ({ dom: signatureDom(help) }) }
          : null;
        view.dispatch({ effects: setSignature.of(tip) });
      }
      destroy() {
        clearTimeout(this.timer);
      }
    },
  );

  return [
    // In the page body, so an editor that scrolls its own content can't clip them.
    tooltips({ parent: document.body }),
    server,
    hover,
    hintField,
    hints,
    editable ? autocompletion({ override: [complete] }) : [],
    editable ? Prec.highest(keymap.of([{ key: "Tab", run: acceptCompletion }])) : [],
    editable ? [signatureField, signature] : [],
    actions.extension,
    keymap.of([
      {
        key: "F12",
        run: (view) => (
          void definition(view, view.state.selection.main.head),
          true
        ),
      },
    ]),
    EditorView.domEventHandlers({
      mousedown(event, view) {
        if (event.button !== 0 || !(event.ctrlKey || event.metaKey))
          return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (pos === null) return false;
        event.preventDefault();
        void actions.link(view, pos).then((hit) => {
          if (!hit) void definition(view, pos);
        });
        return true;
      },
    }),
    EditorView.theme({
      ".cm-tooltip": {
        backgroundColor: "var(--popover)",
        color: "var(--popover-foreground)",
        border: "1px solid var(--border)",
        borderRadius: "6px",
        fontSize: "12px",
      },
      ".cm-lsp-doc": {
        maxWidth: "min(32rem, 90vw)",
        maxHeight: "18rem",
        overflow: "auto",
        padding: "6px 10px",
      },
      ".cm-lsp-doc pre": {
        fontFamily: "var(--font-mono)",
        whiteSpace: "pre-wrap",
        margin: "2px 0",
      },
      ".cm-lsp-doc p": { margin: "6px 0" },
      ".cm-inlay-hint": {
        color: "var(--muted-foreground)",
        fontSize: "0.85em",
        opacity: "0.8",
      },
      ".cm-lsp-doc code": { fontFamily: "var(--font-mono)" },
      ".cm-lsp-doc a": { textDecoration: "underline" },
      ".cm-tooltip-autocomplete > ul": {
        fontFamily: "var(--font-mono)",
        maxHeight: "14rem",
      },
      ".cm-tooltip-autocomplete > ul > li[aria-selected]": {
        backgroundColor: "var(--accent)",
        color: "var(--accent-foreground)",
      },
      ".cm-completionDetail": {
        color: "var(--muted-foreground)",
        marginLeft: "1em",
        fontStyle: "normal",
      },
    }),
  ];
}
