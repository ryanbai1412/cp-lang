import { StreamLanguage, type StreamParser, type StringStream } from '@codemirror/language';
import grammar from '../../../editors/vscode/syntaxes/cp.tmLanguage.json';

// The editor runs the one TextMate grammar (editors/vscode/syntaxes/
// cp.tmLanguage.json, which vscode_sync.rs keeps in step with the compiler)
// itself, line by line: there is no second list of keywords to drift.
// highlight-parity.test.ts holds this runner to Shiki's on the repo's programs.

type Captures = Record<string, { name?: string; patterns?: Raw[] }>;
type Raw = {
  name?: string;
  contentName?: string;
  match?: string;
  begin?: string;
  end?: string;
  include?: string;
  captures?: Captures;
  beginCaptures?: Captures;
  endCaptures?: Captures;
  patterns?: Raw[];
};
type Rule = Raw & { re?: RegExp; endRe?: RegExp; rules?: Rule[] };

const repository = grammar.repository as unknown as Record<string, Raw>;
const compiled = new Map<Raw, Rule>();

function compile(raw: Raw): Rule {
  let rule = compiled.get(raw);
  if (rule) return rule;
  rule = { ...raw };
  compiled.set(raw, rule);
  const src = raw.match ?? raw.begin;
  if (src) rule.re = new RegExp(src, 'dg');
  if (raw.end) rule.endRe = new RegExp(raw.end, 'dg');
  return rule;
}

/** `patterns` with every `include` replaced by what it names. */
function flatten(patterns: Raw[] | undefined, seen = new Set<Raw>()): Rule[] {
  const out: Rule[] = [];
  for (const p of patterns ?? []) {
    const target = p.include === '$self' ? { patterns: grammar.patterns as unknown as Raw[] } : p.include ? repository[p.include.slice(1)] : p;
    if (!target || seen.has(target)) continue;
    if (target.match || target.begin) out.push(compile(target));
    else {
      seen.add(target);
      out.push(...flatten(target.patterns, seen));
      seen.delete(target);
    }
  }
  return out;
}

const rulesOf = (rule: Rule) => (rule.rules ??= flatten(rule.patterns));
const top = flatten(grammar.patterns as unknown as Raw[]);

/** The CodeMirror tag for a scope stack, innermost last. */
export function tagOf(scopes: string[]): string | null {
  for (const s of [...scopes].reverse()) {
    if (s.startsWith('comment')) return 'comment';
    if (s.startsWith('constant.character.escape')) return 'escape';
    if (s.startsWith('string') || s.startsWith('constant.character')) return 'string';
    if (s.startsWith('constant.numeric')) return 'number';
    if (s.startsWith('constant.language')) return 'atom';
    if (s.startsWith('entity.name.function.decorator')) return 'meta';
    if (s.startsWith('keyword.operator.logical')) return 'keyword';
    if (s.startsWith('keyword.operator')) return 'operator';
    if (s.startsWith('keyword') || s.startsWith('storage')) return 'keyword';
    if (s.startsWith('support.type') || s.startsWith('entity.name.type')) return 'typeName';
    if (s.startsWith('entity.name.function') || s.startsWith('support.function')) return 'variableName.function';
    if (s.startsWith('variable') || s.startsWith('entity.name')) return 'variableName';
    if (s.startsWith('punctuation')) return 'punctuation';
  }
  return null;
}

type Segment = { end: number; scopes: string[] };
type Frame = { rule: Rule; scopes: string[]; inner: string[] };
type State = { stack: Frame[]; pending: Segment[] };

const named = (name: string | undefined, m: RegExpExecArray) =>
  name?.replace(/\$(\d+)/g, (_, n) => m[Number(n)] ?? '');

/** The earliest match at or after `from` among `rules` (and `end`, which wins ties). */
function earliest(line: string, from: number, rules: Rule[], end?: RegExp) {
  let best: { m: RegExpExecArray; rule: Rule | null } | null = null;
  const at = (re: RegExp, rule: Rule | null) => {
    re.lastIndex = from;
    const m = re.exec(line);
    if (m && (!best || m.index < best.m.index)) best = { m, rule };
  };
  if (end) at(end, null);
  for (const rule of rules) if (rule.re) at(rule.re, rule);
  return best as { m: RegExpExecArray; rule: Rule | null } | null;
}

/** Segments for match `m` scoped `scopes` + its name, split by its captures. */
function split(line: string, m: RegExpExecArray, scopes: string[], captures: Captures | undefined): Segment[] {
  const start = m.index;
  const end = start + m[0].length;
  const marks = new Array<string[]>(end - start).fill(scopes);
  const sub: [number, number, Rule[], string[]][] = [];
  const indices = m.indices!;
  for (const [k, c] of Object.entries(captures ?? {})) {
    const span = indices[Number(k)];
    if (!span || span[0] === span[1]) continue;
    const inner = c.name ? [...scopes, named(c.name, m)!] : scopes;
    for (let i = span[0]; i < span[1]; i++) marks[i - start] = inner;
    if (c.patterns) sub.push([span[0], span[1], flatten(c.patterns), inner]);
  }
  for (const [from, to, rules, inner] of sub) {
    for (const s of scan(line.slice(0, to), from, rules, inner)) {
      for (let i = s.from; i < s.end; i++) marks[i - start] = s.scopes;
    }
  }
  const out: Segment[] = [];
  for (let i = start; i < end; i++) {
    const s = marks[i - start];
    if (out.length && out[out.length - 1].scopes === s) out[out.length - 1].end = i + 1;
    else out.push({ end: i + 1, scopes: s });
  }
  return out;
}

/** A capture's own patterns over `line` from `from`: single-line matches only. */
function scan(line: string, from: number, rules: Rule[], scopes: string[]) {
  const out: (Segment & { from: number })[] = [];
  for (let pos = from; pos < line.length; ) {
    const hit = earliest(line, pos, rules.filter((r) => r.match));
    if (!hit || !hit.m[0].length) break;
    const rule = hit.rule!;
    let s = hit.m.index;
    for (const seg of split(line, hit.m, rule.name ? [...scopes, named(rule.name, hit.m)!] : scopes, rule.captures)) {
      out.push({ from: s, ...seg });
      s = seg.end;
    }
    pos = hit.m.index + hit.m[0].length;
  }
  return out;
}

/** Tokenizes `line` from `pos` until it has queued at least one segment. */
function advance(line: string, pos: number, state: State) {
  for (let guard = 0; guard < 64; guard++) {
    const frame = state.stack[state.stack.length - 1];
    const rules = frame ? rulesOf(frame.rule) : top;
    const scopes = frame ? frame.inner : [];
    const hit = earliest(line, pos, rules, frame?.rule.endRe);
    if (!hit || hit.m.index > pos) {
      state.pending.push({ end: hit ? hit.m.index : line.length, scopes });
      return;
    }
    const { m, rule } = hit;
    if (rule === null) {
      state.stack.pop();
      state.pending.push(...split(line, m, frame.scopes, frame.rule.endCaptures ?? frame.rule.captures));
    } else if (rule.begin) {
      const outer = rule.name ? [...scopes, named(rule.name, m)!] : scopes;
      state.pending.push(...split(line, m, outer, rule.beginCaptures ?? rule.captures));
      state.stack.push({ rule, scopes: outer, inner: rule.contentName ? [...outer, rule.contentName] : outer });
    } else {
      state.pending.push(...split(line, m, rule.name ? [...scopes, named(rule.name, m)!] : scopes, rule.captures));
    }
    if (state.pending.some((s) => s.end > pos)) return;
  }
  state.pending.push({ end: line.length, scopes: [] });
}

/** Whether `frame` stays open past a line's end: its end can't match there. */
function open(frame: Frame): boolean {
  const end = frame.rule.endRe!;
  end.lastIndex = 0;
  return !end.test('');
}

export const cpParser: StreamParser<State> = {
  name: 'cp',
  startState: () => ({ stack: [], pending: [] }),
  copyState: (s) => ({ stack: [...s.stack], pending: [...s.pending] }),
  token(stream: StringStream, state: State) {
    if (stream.sol()) {
      state.pending = [];
      state.stack = state.stack.filter(open);
    }
    while (state.pending.length && state.pending[0].end <= stream.pos) state.pending.shift();
    if (!state.pending.length) advance(stream.string, stream.pos, state);
    const seg = state.pending.shift()!;
    stream.pos = Math.max(seg.end, stream.pos + 1);
    return tagOf(seg.scopes);
  },
  blankLine(state: State) {
    state.stack = state.stack.filter(open);
  },
  languageData: { commentTokens: { line: '//' }, indentOnInput: /^\s*(elif|else)\b/ },
};

export const cpLanguage = StreamLanguage.define(cpParser);
