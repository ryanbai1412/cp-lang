import cli from '@/content/generated/cli.json';

export interface Command {
  name: string;
  usage: string;
  summary: string;
  help: string | null;
}

export interface Judge {
  name: string;
  default: boolean;
  compiler: string;
  details: string[];
}

export const tools = cli as { version: string; commands: Command[]; judges: Judge[] };

export const groups: { title: string; names: string[] }[] = [
  { title: 'Write and run', names: ['check', 'fix', 'run', 'fmt', 'lint', 'explain'] },
  { title: 'Submit', names: ['build', 'submit', 'judges', 'judge'] },
  { title: 'Test and debug', names: ['new', 'test', 'watch', 'stress', 'bench', 'profile', 'cover'] },
  { title: 'Editors and docs', names: ['lsp', 'dap', 'doc'] },
];
