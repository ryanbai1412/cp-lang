import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHighlighter, type Highlighter } from 'shiki';

const cpGrammar = {
  ...JSON.parse(readFileSync(join(process.cwd(), '../editors/vscode/syntaxes/cp.tmLanguage.json'), 'utf8')),
  name: 'cp',
};

let highlighter: Promise<Highlighter> | null = null;

export async function highlight(code: string, lang: 'cp' | 'cpp' | 'python' | 'java' | 'rust') {
  highlighter ??= createHighlighter({ themes: ['github-light', 'github-dark'], langs: [cpGrammar, 'cpp', 'python', 'java', 'rust'] });
  return (await highlighter).codeToHtml(code, { lang, themes: { light: 'github-light', dark: 'github-dark' } });
}
