import { indentService } from '@codemirror/language';
import { EditorState, StateField, Transaction } from '@codemirror/state';

// Enter keeps the previous line's indent, one level deeper after a line that
// opens a block (ends in `:`, ignoring a trailing comment).
export const blockIndent = indentService.of((cx, pos) => {
  let line = cx.lineAt(pos, -1);
  let text = line.text.slice(0, pos - line.from);
  while (!text.trim() && line.from > 0) {
    line = cx.lineAt(line.from - 1, -1);
    text = line.text;
  }
  const base = /^[ \t]*/.exec(text)![0].replace(/\t/g, '    ').length;
  return /:[ \t]*(\/\/.*)?$/.test(text) ? base + cx.unit : base;
});

// Set by Enter's auto-indent; cleared by any other edit or cursor move.
const autoIndented = StateField.define<boolean>({
  create: () => false,
  update(value, tr) {
    if (!tr.docChanged) return tr.selection ? false : value;
    let inserted = '';
    tr.changes.iterChanges((_fa, _ta, _fb, _tb, text) => (inserted += text.toString()));
    return tr.isUserEvent('input') && /\n[ \t]*$/.test(inserted);
  },
});

// Indentation typed or pasted right after Enter replaces Enter's auto-indent
// rather than adding to it, so code typed with its own indentation lands where
// it was written. Tab still adds a level.
export const typedIndent = [
  autoIndented,
  EditorState.transactionFilter.of((tr) => {
    if (!tr.docChanged || !tr.startState.field(autoIndented)) return tr;
    if (!tr.isUserEvent('input.type') && !tr.isUserEvent('input.paste')) return tr;
    const sel = tr.startState.selection.main;
    const line = tr.startState.doc.lineAt(sel.head);
    if (!sel.empty || sel.head !== line.to || line.text.trim()) return tr;
    let text = '';
    let atCursor = true;
    tr.changes.iterChanges((from, to, _fb, _tb, ins) => {
      atCursor &&= from === sel.head && to === sel.head;
      text += ins.toString();
    });
    if (!atCursor || !/^[ \t]/.test(text)) return tr;
    return {
      changes: { from: line.from, to: line.to, insert: text },
      selection: { anchor: line.from + text.length },
      userEvent: tr.annotation(Transaction.userEvent),
      scrollIntoView: true,
    };
  }),
];
