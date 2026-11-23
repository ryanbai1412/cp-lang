import { insertNewlineAndIndent, indentLess, indentMore } from '@codemirror/commands';
import { IndentContext, indentService, indentUnit } from '@codemirror/language';
import { EditorState, type Transaction } from '@codemirror/state';
import { describe, expect, it } from 'vitest';
import { blockIndent, typedIndent } from './indent';

const start = (doc: string) =>
  EditorState.create({ doc, selection: { anchor: doc.length }, extensions: [indentUnit.of('    '), blockIndent, typedIndent] });

function run(state: EditorState, command: (t: { state: EditorState; dispatch: (tr: Transaction) => void }) => boolean) {
  let next = state;
  command({ state, dispatch: (tr) => (next = tr.state) });
  return next;
}

const typeLines = (state: EditorState, lines: string[]) =>
  lines.reduce((s, l, i) => type(i ? run(s, insertNewlineAndIndent) : s, l), state);

const type = (state: EditorState, text: string, event = 'input.type') =>
  [...(event === 'input.type' ? text : [text])].reduce(
    (s, ch) => s.update({ changes: { from: s.selection.main.head, insert: ch }, selection: { anchor: s.selection.main.head + ch.length }, userEvent: event }).state,
    state,
  );

describe('typed indentation after Enter', () => {
  it('replaces the auto-indent when the code brings its own', () => {
    const s = type(run(start('for i in 0..n:'), insertNewlineAndIndent), '    print(i)');
    expect(s.doc.toString()).toBe('for i in 0..n:\n    print(i)');
  });

  it('replaces it for pasted lines too', () => {
    const s = type(run(start('if a:'), insertNewlineAndIndent), '    x += 1\n    y += 1', 'input.paste');
    expect(s.doc.toString()).toBe('if a:\n    x += 1\n    y += 1');
  });

  it('keeps the auto-indent for code typed without indentation', () => {
    const s = type(run(start('if a:'), insertNewlineAndIndent), 'x += 1');
    expect(s.doc.toString()).toBe('if a:\n    x += 1');
  });

  it('leaves Tab adding a level', () => {
    const s = run(run(start('if a:'), insertNewlineAndIndent), indentMore);
    expect(s.doc.toString()).toBe('if a:\n        ');
  });
});

describe('Enter after a block', () => {
  it('dedents after return, break and continue', () => {
    for (const kw of ['return x', 'break', 'continue']) {
      const s = run(start(`fn f():\n    while a:\n        ${kw}`), insertNewlineAndIndent);
      expect(s.doc.toString()).toBe(`fn f():\n    while a:\n        ${kw}\n    `);
    }
  });

  it('lines elif and else up with their if', () => {
    const s = start('if a:\n    x += 1\n    else:');
    expect(s.facet(indentService)[0](new IndentContext(s), s.doc.line(3).from)).toBe(0);
  });

  it('puts an unindented line at column 0 once the typist brings their own indentation', () => {
    const s = typeLines(start(''), ['while n % 2 == 0:', '    n /= 2', '    k += 1', 'print(k, n)']);
    expect(s.doc.toString()).toBe('while n % 2 == 0:\n    n /= 2\n    k += 1\nprint(k, n)');
  });

  it('keeps the auto-indent for a typist who relies on it', () => {
    const s = typeLines(start(''), ['while a:', 'x += 1', 'y += 1']);
    expect(s.doc.toString()).toBe('while a:\n    x += 1\n    y += 1');
  });

  it('leaves Shift-Tab removing a level', () => {
    const s = run(run(start('while a:'), insertNewlineAndIndent), indentLess);
    expect(s.doc.toString()).toBe('while a:\n');
  });
});
