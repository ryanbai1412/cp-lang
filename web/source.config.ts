import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
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
          attributes: [attr('source', child.value + '\n'), attr('stdin', stdin), attr('status', 'runs')],
          children: [],
        } as Node;
      } else visit(child);
    });
  };
  return (tree: Node) => visit(tree);
}

export default defineConfig({
  mdxOptions: {
    remarkPlugins: [remarkRun],
    rehypeCodeOptions: {
      themes: { light: 'github-light', dark: 'github-dark' },
      langs: [{ ...cpGrammar, name: 'cp', aliases: ['cplang'] }, 'cpp', 'python', 'java', 'rust', 'sh', 'json'],
    },
  },
});
