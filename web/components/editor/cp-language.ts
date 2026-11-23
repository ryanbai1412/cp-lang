import { StreamLanguage, type StreamParser, type StringStream } from '@codemirror/language';

const keywords = new Set([
  'if', 'elif', 'else', 'for', 'while', 'in', 'match', 'return', 'break', 'continue',
  'assert', 'yield', 'and', 'or', 'not', 'as', 'use', 'unchecked',
]);
const declarations = new Set([
  'let', 'var', 'fn', 'struct', 'enum', 'domain', 'type', 'trait', 'impl', 'mut', 'lends', 'pub', 'const',
]);
// As the TextMate grammar (editors/vscode/syntaxes/cp.tmLanguage.json) reads
// them, which highlight-parity.test.ts checks: the literal words, and the
// words that are keywords only in one place.
const atoms = new Set(['true', 'false', 'None', 'none']);

function word(stream: StringStream, prev: string): string | null {
  const w = stream.current();
  const rest = stream.string.slice(stream.pos);
  if (prev === '.') return w === 'raw' ? 'keyword' : null;
  if ((prev === '(' || prev === ',' || prev === ';') && /^\s*:(?!:)/.test(rest)) return 'variableName';
  if (w === 'yields' && prev === '->') return 'keyword';
  if (w === 'do' && /^:\s*(?:$|\/\/)/.test(rest)) return 'keyword';
  if (w === 'law' && !stream.string.slice(0, stream.start).trim() && /^\s+[a-z_]\w*\s*\(/.test(rest)) return 'keyword';
  if (keywords.has(w) || declarations.has(w)) return 'keyword';
  return null;
}

type State = { inString: boolean; prev: string };

function token(stream: StringStream, state: State): string | null {
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
    const tag = word(stream, state.prev);
    if (tag) return tag;
    const w = stream.current();
    if (atoms.has(w)) return 'atom';
    if (/^[A-Z]/.test(w)) return 'typeName';
    if (stream.peek() === '(') return 'variableName.function';
    return 'variableName';
  }
  if (stream.match(/^(?:\.\.=|\.\.|->|=>|\?\?|[-+*/%<>=!&|^~?]=?)/)) return 'operator';
  stream.next();
  return 'punctuation';
}

export const cpParser: StreamParser<State> = {
  name: 'cp',
  startState: () => ({ inString: false, prev: '' }),
  token(stream, state) {
    const tag = token(stream, state);
    if (tag !== null || stream.current().trim()) state.prev = stream.current();
    return tag;
  },
  languageData: { commentTokens: { line: '//' }, indentOnInput: /^\s*(elif|else)\b/ },
};

export const cpLanguage = StreamLanguage.define(cpParser);
