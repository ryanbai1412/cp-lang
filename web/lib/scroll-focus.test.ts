// site/R14 (handbook/guidelines/site-checklist.md): a region that scrolls must take keyboard focus (axe
// scrollable-region-focusable), so every element whose own class scrolls
// carries tabIndex={0}. Editors get focus from CodeMirror (code-editor.tsx).
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const web = join(__dirname, '..');
const skip = ['components/ui/', 'components/editor/code-editor.tsx'];
const files = (dir: string): string[] =>
  readdirSync(join(web, dir), { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? files(join(dir, e.name)) : e.name.endsWith('.tsx') ? [join(dir, e.name)] : [],
  );

export function unfocusable(src: string): number[] {
  const out: number[] = [];
  for (const m of src.matchAll(/<\w+\b[^>]*?(?:className=(?:"[^"]*"|\{[^}]*\}))[^>]*>/gs)) {
    const tag = m[0];
    if (!/(?<![\w&:-])overflow-(?:[xy]-)?auto\b|className=\{box\}/.test(tag) || /tabIndex=/.test(tag)) continue;
    out.push(src.slice(0, m.index).split('\n').length);
  }
  return out;
}

describe('scrolling regions take keyboard focus', () => {
  it('finds a scrolling tag without tabIndex', () => {
    expect(unfocusable('<pre className="overflow-x-auto p-3">x</pre>')).toEqual([1]);
    expect(unfocusable('<pre tabIndex={0} className="overflow-x-auto">x</pre>')).toEqual([]);
    expect(unfocusable('<div className="prose [&_pre]:overflow-x-auto">x</div>')).toEqual([]);
  });
  it('holds for every page and component', () => {
    const bad = [...files('app'), ...files('components')]
      .filter((f) => !skip.some((s) => f.startsWith(s)))
      .flatMap((f) => unfocusable(readFileSync(join(web, f), 'utf8')).map((l) => `${f}:${l}`));
    expect(bad).toEqual([]);
  });
});
