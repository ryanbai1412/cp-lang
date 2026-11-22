import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export interface Test {
  input: string;
  expected: string;
  sample?: boolean;
}

export interface Problem {
  id: string;
  starter: string;
  reference: string;
  tests: Test[];
}

export function problem(id: string): Problem {
  const dir = join(process.cwd(), 'content/practice', id);
  const read = (name: string) => readFileSync(join(dir, name), 'utf8');
  return { id, starter: read('starter.cp'), reference: read('ref.cp'), tests: JSON.parse(read('tests.json')) };
}
