// The editors' highlighter (cp-language.ts) colours each word as the TextMate
// grammar, which vscode_sync.rs keeps in step with the compiler, colours it on
// the repo's programs (site/R16).
import { StringStream } from '@codemirror/language';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHighlighter } from 'shiki';
import { expect, test } from 'vitest';
import { cpParser } from './cp-language';

const repo = join(__dirname, '../../..');
// Where the grammar is the one that's wrong, sent to tools (who own it): it
// doesn't read `domain` in `fn f<domain I>()` or `yields` in `-> yields T` as
// a keyword.
const known = new Set(['domain: grammar other, editor keyword', 'yields: grammar other, editor keyword']);
type Class = 'keyword' | 'atom' | 'string' | 'comment' | 'number' | 'other';

function grammarClass(scopes: string[]): Class {
  for (const s of [...scopes].reverse()) {
    if (s.startsWith('comment')) return 'comment';
    if (s.startsWith('string') || s.startsWith('constant.character')) return 'string';
    if (s.startsWith('constant.numeric')) return 'number';
    if (s.startsWith('constant.language')) return 'atom';
    if (s.startsWith('keyword.control') || s.startsWith('keyword.operator.logical') || s.startsWith('storage') || s.startsWith('keyword.other')) return 'keyword';
  }
  return 'other';
}

function editorClass(tag: string | null): Class {
  if (tag === 'keyword') return 'keyword';
  if (tag === 'atom') return 'atom';
  if (tag === 'string' || tag === 'comment' || tag === 'number') return tag;
  return 'other';
}

test('the editor colours words as the grammar does', async () => {
  const grammar = { ...JSON.parse(readFileSync(join(repo, 'editors/vscode/syntaxes/cp.tmLanguage.json'), 'utf8')), name: 'cp' };
  const hl = await createHighlighter({ themes: ['github-light'], langs: [grammar] });
  const files = execFileSync('git', ['ls-files', 'std/*.cp', 'corpus/problems/*/port.cp'], { cwd: repo, encoding: 'utf8' }).split('\n').filter(Boolean);
  const differ = new Map<string, number>();
  const example = new Map<string, string>();
  let words = 0;
  for (const file of files) {
    const code = readFileSync(join(repo, file), 'utf8');
    const lines = hl.codeToTokens(code, { lang: grammar.name, theme: 'github-light', includeExplanation: true }).tokens;
    const state = cpParser.startState!(2);
    code.split('\n').forEach((line, i) => {
      const spans: { at: number; scopes: string[] }[] = [];
      let at = 0;
      for (const t of lines[i] ?? []) for (const e of t.explanation ?? []) {
        spans.push({ at, scopes: e.scopes.map((s) => s.scopeName) });
        at += e.content.length;
      }
      const stream = new StringStream(line, 4, 2);
      while (!stream.eol()) {
        const tag = cpParser.token(stream, state);
        const text = stream.current();
        const start = stream.start;
        stream.start = stream.pos;
        if (!/^[A-Za-z_]\w*$/.test(text)) continue;
        const span = spans.findLast((s) => s.at <= start);
        if (!span) continue;
        const want = grammarClass(span.scopes);
        const got = editorClass(tag);
        if (want === 'string' || want === 'comment') continue;
        words++;
        if (want === got) continue;
        const key = `${text}: grammar ${want}, editor ${got}`;
        differ.set(key, (differ.get(key) ?? 0) + 1);
        if (!example.has(key)) example.set(key, `${file}:${i + 1}: ${line.trim()}`);
      }
    });
  }
  const fresh = [...differ].filter(([k]) => !known.has(k)).sort((a, b) => b[1] - a[1]).map(([k, n]) => `${k} (${n}x, e.g. ${example.get(k)})`);
  console.log(`highlight parity: ${files.length} files, ${words} words, ${differ.size} kinds differ (${known.size} known)`);
  expect(fresh, 'colour these as the grammar does').toEqual([]);
  expect([...known].filter((k) => !differ.has(k)), 'fixed now: delete from known').toEqual([]);
}, 300_000);
