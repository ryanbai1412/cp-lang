import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

// Error pages come from `cpl explain --json`: the registry in
// crates/cpl-diag/src/registry.rs, whose triggers and fixes
// crates/cpl-cli/tests/catalogue.rs holds to the compiler.
export type ErrorCode = {
  code: string;
  title: string;
  summary: string;
  explanation: string;
  trigger: string;
  fix: string;
  mode: string;
  input: string;
  lint_name: string | null;
};

const repo = join(process.cwd(), '..');
let cache: ErrorCode[] | null = null;

export function codes(): ErrorCode[] {
  if (!cache) {
    const cpl =
      process.env.CPL ?? ['target/release/cpl', 'target/debug/cpl'].map((p) => join(repo, p)).find(existsSync) ?? 'cpl';
    cache = JSON.parse(execFileSync(cpl, ['explain', '--json'], { encoding: 'utf8', maxBuffer: 16 << 20 })) as ErrorCode[];
  }
  return cache;
}

// The families of spec/diagnostics.md. Lints (`L`) have their own pages.
export const families = [
  ['E00', 'Lexing'],
  ['E01', 'Parsing'],
  ['E02', 'Names'],
  ['E03', 'Types and domains'],
  ['E04', 'Mutation'],
  ['E05', 'Unsupported and removed'],
  ['E06', 'Lending'],
  ['E07', 'Traits and generics'],
  ['E08', 'Contexts, checks and fresh types'],
  ['E09', 'Numbers and layout'],
  ['W', 'Warnings'],
  ['R', 'Runtime traps'],
] as const;

export const familyOf = (code: string) => families.find(([prefix]) => code.startsWith(prefix));
