'use client';

import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
import { bracketMatching, HighlightStyle, indentUnit, syntaxHighlighting } from '@codemirror/language';
import { type Diagnostic as CmDiagnostic, linter, lintGutter } from '@codemirror/lint';
import { EditorState } from '@codemirror/state';
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
import { cpl, type Diagnostic } from '@/lib/cpl';
import { cn } from '@/lib/utils';
import { cpLanguage } from './cp-language';

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

export function toCmDiagnostics(diagnostics: Diagnostic[], length: number): CmDiagnostic[] {
  return diagnostics.map((d) => ({
    from: Math.min(d.start, length),
    to: Math.min(Math.max(d.end, d.start), length),
    severity: d.severity === 'note' ? 'info' : d.severity,
    source: d.code,
    message: [d.message, d.help ? `help: ${d.help}` : null].filter(Boolean).join('\n'),
  }));
}

const checkLinter = linter(
  async (view) => {
    const source = view.state.doc.toString();
    try {
      const outcome = await cpl.check(source);
      return toCmDiagnostics(outcome.diagnostics, source.length);
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
}

export function CodeEditor({ value, onChange, onRun, readOnly, live = true, className, minHeight }: CodeEditorProps) {
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
        cpLanguage,
        syntaxHighlighting(highlight),
        theme,
        EditorState.readOnly.of(!!readOnly),
        keymap.of([
          { key: 'Mod-Enter', run: () => (callbacks.current.onRun?.(), true) },
          indentWithTab,
          ...defaultKeymap,
          ...historyKeymap,
        ]),
        ...(live && !readOnly ? [checkLinter, lintGutter()] : []),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) callbacks.current.onChange?.(u.state.doc.toString());
        }),
        minHeight ? EditorView.theme({ '.cm-content, .cm-gutter': { minHeight } }) : [],
      ],
    });
    view.current = new EditorView({ state, parent: host.current! });
    return () => view.current?.destroy();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [readOnly, live]);

  useEffect(() => {
    const v = view.current;
    if (v && v.state.doc.toString() !== value) {
      v.dispatch({ changes: { from: 0, to: v.state.doc.length, insert: value } });
    }
  }, [value]);

  return <div ref={host} className={cn('overflow-auto', className)} />;
}
