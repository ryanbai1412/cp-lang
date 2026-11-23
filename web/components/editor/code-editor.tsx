'use client';

import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
import { bracketMatching, HighlightStyle, indentOnInput, indentUnit, syntaxHighlighting } from '@codemirror/language';
import { blockIndent, typedIndent } from './indent';
import { type Diagnostic as CmDiagnostic, linter, lintGutter } from '@codemirror/lint';
import { EditorState, StateField, Transaction } from '@codemirror/state';
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from '@codemirror/view';
import { tags as t } from '@lezer/highlight';
import { useEffect, useRef } from 'react';
import { cpl, type Diagnostic, started } from '@/lib/cpl';
import { cn } from '@/lib/utils';
import { cpLanguage } from './cp-language';
import { intelligence } from './intelligence';

const highlight = HighlightStyle.define([
  { tag: t.keyword, color: 'var(--cp-keyword)' },
  { tag: [t.atom, t.bool], color: 'var(--cp-atom)' },
  { tag: t.number, color: 'var(--cp-number)' },
  { tag: t.string, color: 'var(--cp-string)' },
  { tag: t.comment, color: 'var(--muted-foreground)', fontStyle: 'italic' },
  { tag: t.meta, color: 'var(--cp-meta)' },
  { tag: t.typeName, color: 'var(--cp-type)' },
  { tag: t.function(t.variableName), color: 'var(--cp-function)' },
  { tag: t.operator, color: 'var(--cp-operator)' },
]);

const theme = EditorView.theme({
  '&': { fontSize: '13px', backgroundColor: 'transparent', height: '100%' },
  '.cm-content': { fontFamily: 'var(--font-mono)', padding: '12px 0', caretColor: 'var(--foreground)' },
  '.cm-scroller': { fontFamily: 'var(--font-mono)', lineHeight: '1.6' },
  '.cm-gutters': { backgroundColor: 'transparent', border: 'none', color: 'var(--muted-foreground)' },
  '.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'color-mix(in oklab, var(--muted) 60%, transparent)' },
  '&.cm-focused': { outline: 'none' },
  '.cm-selectionBackground, &.cm-focused .cm-selectionBackground': {
    backgroundColor: 'color-mix(in oklab, var(--ring) 35%, transparent) !important',
  },
});

const shift = (d: Diagnostic, by: number): Diagnostic =>
  by === 0
    ? d
    : {
        ...d,
        start: d.start + by,
        end: d.end + by,
        fixes: d.fixes.map((f) => ({ ...f, edits: f.edits.map((e) => ({ ...e, start: e.start + by, end: e.end + by })) })),
      };

export function toCmDiagnostics(diagnostics: Diagnostic[], length: number): CmDiagnostic[] {
  return diagnostics.map((d) => ({
    from: Math.min(d.start, length),
    to: Math.min(Math.max(d.end, d.start), length),
    severity: d.severity === 'note' ? 'info' : d.severity,
    source: d.code,
    message: [d.message, d.help ? `help: ${d.help}` : null].filter(Boolean).join('\n'),
    actions: [
      ...d.fixes.map((f) => ({
        name: f.title,
        apply: (view: EditorView) =>
          view.dispatch({ changes: f.edits.map((e) => ({ from: e.start, to: e.end, insert: e.text })) }),
      })),
      ...(/^[LEWR]\d{4}$/.test(d.code)
        ? [{ name: 'Why?', apply: () => window.open(`/${d.code[0] === 'L' ? 'lints' : 'errors'}/${d.code}`, '_blank', 'noopener') }]
        : []),
    ],
  }));
}

// True once the user has typed: until then an editor only checks if asked to
// on load or if the compiler is already loaded, so showing code costs no wasm.
const edited = StateField.define<boolean>({
  create: () => false,
  update: (was, tr) => was || (tr.docChanged && tr.annotation(Transaction.userEvent) !== undefined),
});

// `prefix` is code run before the editor's text (a collapsed setup): it is
// checked with it, and only diagnostics in the editor's text are shown.
const checkLinter = (onLoad: boolean, prefix = '') => linter(
  async (view) => {
    if (!onLoad && !started() && !view.state.field(edited)) return [];
    const shown = view.state.doc.toString();
    const head = prefix ? `${prefix}\n` : '';
    const source = head + shown;
    try {
      const outcome = await cpl.check(source);
      // Lints wait for a program that checks, as `cpl lint` does.
      const lints = outcome.ok ? (await cpl.lint(source)).diagnostics : [];
      return toCmDiagnostics(
        [...outcome.diagnostics, ...lints].map((d) => shift(d, -head.length)).filter((d) => d.start >= 0),
        shown.length,
      );
    } catch {
      return [];
    }
  },
  { delay: 400 },
);

export interface CodeEditorProps {
  value: string;
  onChange?: (value: string) => void;
  onRun?: () => void;
  readOnly?: boolean;
  live?: boolean;
  className?: string;
  minHeight?: string;
  /** Check the initial text at once, loading the compiler; otherwise wait for an edit or a run. */
  checkOnLoad?: boolean;
  /** Code run before `value`, checked with it but not shown. */
  prefix?: string;
  /** What the editor holds, for screen readers (site/R14). */
  label?: string;
}

export function CodeEditor({ value, onChange, onRun, readOnly, live = true, className, minHeight, checkOnLoad = false, prefix = '', label }: CodeEditorProps) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const callbacks = useRef({ onChange, onRun });
  callbacks.current = { onChange, onRun };

  useEffect(() => {
    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        drawSelection(),
        history(),
        bracketMatching(),
        indentUnit.of('    '),
        blockIndent,
        typedIndent,
        indentOnInput(),
        cpLanguage,
        syntaxHighlighting(highlight),
        theme,
        EditorState.readOnly.of(!!readOnly),
        EditorView.contentAttributes.of({ 'aria-label': label ?? (readOnly ? 'CP Lang example' : 'CP Lang source') }),
        keymap.of([
          { key: 'Mod-Enter', run: () => (callbacks.current.onRun?.(), true) },
          indentWithTab,
          ...defaultKeymap,
          ...historyKeymap,
        ]),
        ...(live && !readOnly ? [edited, checkLinter(checkOnLoad, prefix), lintGutter()] : []),
        ...(live ? [intelligence(prefix, !readOnly)] : []),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) callbacks.current.onChange?.(u.state.doc.toString());
        }),
        minHeight ? EditorView.theme({ '.cm-content, .cm-gutter': { minHeight } }) : [],
      ],
    });
    view.current = new EditorView({ state, parent: host.current! });
    return () => view.current?.destroy();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [readOnly, live, checkOnLoad, prefix]);

  useEffect(() => {
    const v = view.current;
    if (v && v.state.doc.toString() !== value) {
      v.dispatch({ changes: { from: 0, to: v.state.doc.length, insert: value } });
    }
  }, [value]);

  return <div ref={host} className={cn('overflow-auto', className)} />;
}
