import { existsSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { defineConfig, defineDocs } from 'fumadocs-mdx/config';

const cpGrammar = JSON.parse(
  readFileSync(resolve(process.cwd(), '../editors/vscode/syntaxes/cp.tmLanguage.json'), 'utf8'),
);

export const docs = defineDocs({ dir: 'content/docs' });
export const learn = defineDocs({ dir: 'content/learn' });

type Node = { type: string; lang?: string; meta?: string | null; value?: string; children?: Node[] };

// A fence written ```cp run input="..." becomes a live <Runner>; the snippet gate reads the same fence.
function remarkRun() {
  const visit = (node: Node) => {
    node.children?.forEach((child, i) => {
      if (child.type === 'code' && child.lang === 'cp' && /(^|\s)run(\s|$)/.test(child.meta ?? '')) {
        const stdin = /\binput="([^"]*)"/.exec(child.meta ?? '')?.[1]?.replace(/\\n/g, '\n') ?? '';
        const attr = (name: string, value: string) => ({ type: 'mdxJsxAttribute', name, value });
        node.children![i] = {
          type: 'mdxJsxFlowElement',
          name: 'Runner',
          attributes: [attr('source', child.value + '\n'), attr('stdin', stdin), attr('status', /\brejects=[ER]\d{4}\b/.test(child.meta ?? '') ? 'rejects' : 'runs')],
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
            attributes: [attr('source', child.value + '\n'), attr('stdin', stdin), ...(g.ran ? [attr('ran', 'true')] : [])],
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
    remarkPlugins: [remarkLive, remarkRun],
    rehypeCodeOptions: {
      themes: { light: 'github-light', dark: 'github-dark' },
      langs: [{ ...cpGrammar, name: 'cp', aliases: ['cplang'] }, 'cpp', 'python', 'java', 'rust', 'sh', 'json'],
    },
  },
});
