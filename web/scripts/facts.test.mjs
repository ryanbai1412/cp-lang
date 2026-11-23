import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { contradictions } from './facts-rules.mjs';

const facts = JSON.parse(readFileSync(new URL('../lib/facts.json', import.meta.url), 'utf8'));
const bad = (text) => contradictions(text, facts).map((c) => c.msg.split(' `')[0]);

describe('facts gate', () => {
  it('accepts the facts as stated', () => {
    expect(bad(`${facts.install.sh}   # Linux\n${facts.install.ps1}\n${facts.edgeVersion}\nsaved to solution.tests/`)).toEqual([]);
  });
  it('rejects a different install one-liner', () => {
    expect(bad('curl -fsSL https://cp-lang.org/install.sh | sh -s -- --edge')).toEqual(['install.sh one-liner']);
    expect(bad('& ([scriptblock]::Create((irm https://cp-lang.org/install.ps1)))')).toEqual(['install.ps1 one-liner']);
  });
  it('rejects a stale version string', () => {
    expect(bad('prints `cpl 0.3.0-dev+<commit>`')).toEqual(['version string', 'version string']);
    expect(bad('cpl 0.2.0+abc')).toEqual(['version string']);
  });
  it('rejects the old saved-case folder', () => {
    expect(bad('saved to solution.cp.tests/')).toEqual(['saved-case folder (cpl saves to <file>.tests/)']);
  });
});
