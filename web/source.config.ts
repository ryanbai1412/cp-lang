import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { defineConfig, defineDocs } from 'fumadocs-mdx/config';

const cpGrammar = JSON.parse(
  readFileSync(resolve(process.cwd(), '../editors/vscode/syntaxes/cp.tmLanguage.json'), 'utf8'),
);

export const docs = defineDocs({ dir: 'content/docs' });

export default defineConfig({
  mdxOptions: {
    rehypeCodeOptions: {
      themes: { light: 'github-light', dark: 'github-dark' },
      langs: [{ ...cpGrammar, name: 'cp', aliases: ['cplang'] }, 'cpp', 'python', 'java', 'rust', 'sh', 'json'],
    },
  },
});
