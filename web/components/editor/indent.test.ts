import { insertNewlineAndIndent, indentMore } from '@codemirror/commands';
import { indentUnit } from '@codemirror/language';
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
