import { StreamLanguage, type StreamParser } from '@codemirror/language';

const keywords = new Set([
  'if', 'elif', 'else', 'for', 'while', 'in', 'match', 'return', 'break', 'continue',
  'assert', 'yield', 'and', 'or', 'not', 'as', 'use', 'unchecked', 'todo',
]);
const declarations = new Set([
  'let', 'var', 'fn', 'struct', 'enum', 'domain', 'type', 'trait', 'impl', 'mut', 'lends', 'pub', 'const',
]);
const atoms = new Set(['true', 'false', 'None', 'Some']);

const parser: StreamParser<{ inString: boolean }> = {
  name: 'cp',
  startState: () => ({ inString: false }),
  token(stream, state) {
    if (state.inString) {
      while (!stream.eol()) {
        const ch = stream.next();
        if (ch === '\\') stream.next();
        else if (ch === '"') {
          state.inString = false;
          break;
        }
      }
      return 'string';
    }
    if (stream.eatSpace()) return null;
    if (stream.match('//')) {
      stream.skipToEnd();
      return 'comment';
    }
    if (stream.peek() === '"') {
      stream.next();
      state.inString = true;
      return 'string';
    }
    if (stream.match(/^'(?:\\.|[^'\\])'/)) return 'string';
    if (stream.match(/^@!?[A-Za-z_]\w*/)) return 'meta';
    if (stream.match(/^0[xX][0-9a-fA-F_]+|^0[bB][01_]+|^\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?/))
      return 'number';
    if (stream.match(/^[A-Za-z_]\w*/)) {
      const word = stream.current();
      if (keywords.has(word)) return 'keyword';
      if (declarations.has(word)) return 'keyword';
      if (atoms.has(word)) return 'atom';
      if (/^[A-Z]/.test(word)) return 'typeName';
      if (stream.peek() === '(') return 'variableName.function';
      return 'variableName';
    }
    if (stream.match(/^(?:\.\.=|\.\.|->|=>|\?\?|[-+*/%<>=!&|^~?]=?)/)) return 'operator';
    stream.next();
    return 'punctuation';
  },
  languageData: { commentTokens: { line: '//' }, indentOnInput: /^\s*(elif|else)\b/ },
};

export const cpLanguage = StreamLanguage.define(parser);
