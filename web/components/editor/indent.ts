import { indentService } from '@codemirror/language';

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
