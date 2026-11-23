import { existsSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { defineConfig, defineDocs } from 'fumadocs-mdx/config';
import { decisionHrefs } from './lib/decisions';
import rehypeKatex from 'rehype-katex';
import remarkMath from 'remark-math';

const cpGrammar = JSON.parse(
  readFileSync(resolve(process.cwd(), '../editors/vscode/syntaxes/cp.tmLanguage.json'), 'utf8'),
);

export const docs = defineDocs({ dir: 'content/docs' });
export const learn = defineDocs({ dir: 'content/learn' });

type Node = { type: string; lang?: string; meta?: string | null; value?: string; children?: Node[] };

// A `//---` line splits a fence into a setup, shown collapsed, and the code shown.
function sourceAttrs(value = '') {
  const attr = (name: string, value: string) => ({ type: 'mdxJsxAttribute', name, value });
  const i = value.indexOf('\n//---\n');
  return i < 0 ? [attr('source', value + '\n')] : [attr('setup', value.slice(0, i)), attr('source', value.slice(i + 7) + '\n')];
}

// A fence written ```cp run input="..." becomes a live <Runner>; the snippet gate reads the same fence.
// "decision 128", "decisions 53 and 61", and links into decisions.md on
// GitHub: each number that names an entry links to its page on /decisions.
const decided = decisionHrefs();
const cite = /\b([Dd]ecisions?) (\d+(?:\.\d+)?(?:(?:, | and | or )\d+(?:\.\d+)?)*)/g;
const github = /decisions\.md#(\d+)(?:-|$)/;
type MdNode = { type: string; value?: string; url?: string; children?: MdNode[] };
function remarkDecisions() {
  const walk = (node: MdNode) => {
    const gh = node.type === 'link' && github.exec(node.url ?? '');
    if (gh && decided.has(gh[1])) node.url = decided.get(gh[1]);
    if (!node.children || node.type === 'link' || node.type === 'heading') return;
    node.children = node.children.flatMap((child) => {
      if (child.type !== 'text' || !child.value) {
        walk(child);
        return [child];
      }
      const out: MdNode[] = [];
      let last = 0;
      for (const m of child.value.matchAll(cite)) {
        out.push({ type: 'text', value: child.value.slice(last, m.index) + m[1] + ' ' });
        for (const piece of m[2].split(/(, | and | or )/)) {
          out.push(
            decided.has(piece)
              ? { type: 'link', url: decided.get(piece), children: [{ type: 'text', value: piece }] }
              : /^\d/.test(piece)
                ? fail(piece)
                : { type: 'text', value: piece },
          );
        }
        last = m.index + m[0].length;
      }
      if (!out.length) return [child];
      out.push({ type: 'text', value: child.value.slice(last) });
      return out;
    });
  };
  let fail = (n: string): MdNode => {
    throw new Error(`decision ${n} is cited but decisions.md has no entry ${n}`);
  };
  return (tree: MdNode, file: { path?: string }) => {
    fail = (n) => {
      throw new Error(`${file.path}: decision ${n} is cited but decisions.md has no entry ${n}`);
    };
    walk(tree);
  };
}

function remarkRun() {
  const visit = (node: Node) => {
    node.children?.forEach((child, i) => {
      if (child.type === 'code' && child.lang === 'cp' && /(^|\s)run(\s|$)/.test(child.meta ?? '')) {
        const stdin = /\binput="([^"]*)"/.exec(child.meta ?? '')?.[1]?.replace(/\\n/g, '\n') ?? '';
        const attr = (name: string, value: string) => ({ type: 'mdxJsxAttribute', name, value });
        node.children![i] = {
          type: 'mdxJsxFlowElement',
          name: 'Runner',
          attributes: [...sourceAttrs(child.value), attr('stdin', stdin), attr('status', /\brejects=[ER]\d{4}\b/.test(child.meta ?? '') ? 'rejects' : 'runs'), ...(/(^|\s)compact(\s|$)/.test(child.meta ?? '') ? [{ type: 'mdxJsxAttribute', name: 'compact', value: null }] : [])],
          children: [],
        } as Node;
      } else visit(child);
    });
  };
  return (tree: Node) => visit(tree);
}

// Every other ```cp fence the snippet gate measured as `runs` gets a Run button and an
// "Open in playground" link (<LiveCode>), keeping its highlighted block. A fence's gate id
// is its number among the file's fences as gate.mjs matches them, found here by its text.
type Gate = Record<string, { status: string; ran?: boolean }>;
let gate: Gate | null = null;
function remarkLive() {
  return (tree: Node, file: { path: string; value: unknown }) => {
    const statusFile = resolve(process.cwd(), '.gate/status.json');
    gate ??= existsSync(statusFile) ? (JSON.parse(readFileSync(statusFile, 'utf8')) as Gate) : {};
    const page = relative(process.cwd(), file.path);
    const fences = [...String(file.value).matchAll(/^```cp((?:[ \t][^\n]*)?)\n([\s\S]*?)^```/gm)].map((m) => m[2]);
    let next = 0;
    const visit = (node: Node) => {
      node.children?.forEach((child, i) => {
        if (child.type === 'code' && child.lang === 'cp') {
          const k = fences.indexOf(`${child.value}\n`, next);
          if (k < 0) return;
          next = k + 1;
          const g = gate![`${page}#${k + 1}`];
          if (g?.status !== 'runs' || /(^|\s)run(\s|$)/.test(child.meta ?? '')) return;
          const stdin = /\binput="([^"]*)"/.exec(child.meta ?? '')?.[1]?.replace(/\\n/g, '\n') ?? '';
          const attr = (name: string, value: string) => ({ type: 'mdxJsxAttribute', name, value });
          node.children![i] = {
            type: 'mdxJsxFlowElement',
            name: 'LiveCode',
            attributes: [...sourceAttrs(child.value), attr('stdin', stdin), ...(g.ran ? [attr('ran', 'true')] : [])],
            children: [child],
          } as Node;
        } else visit(child);
      });
    };
    visit(tree);
  };
}

export default defineConfig({
  mdxOptions: {
    remarkPlugins: [[remarkMath, { singleDollarTextMath: false }], remarkLive, remarkRun, remarkDecisions],
    rehypePlugins: (v) => [rehypeKatex, ...v],
    rehypeCodeOptions: {
      themes: { light: 'github-light-default', dark: 'github-dark-default' },
      langs: [{ ...cpGrammar, name: 'cp', aliases: ['cplang'] }, 'cpp', 'python', 'java', 'rust', 'sh', 'json'],
    },
  },
});
