import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const run = (suites, env = {}) => {
  const r = spawnSync(process.execPath, [fileURLToPath(new URL('./run.mjs', import.meta.url)), ...suites], {
    encoding: 'utf8',
    env: { ...process.env, BASE: 'http://127.0.0.1:9', CDP: 'unused', EXIT_GRACE_MS: '300', ...env },
  });
  return { status: r.status, out: r.stdout + r.stderr };
};

describe('tests/run.mjs', () => {
  it('passes a suite that ends with done()', () => {
    expect(run(['fixtures/pass.mjs'])).toMatchObject({ status: 0 });
  });
  it('fails, by name, a suite still running after its result line', () => {
    const r = run(['fixtures/hang-after-result.mjs']);
    expect(r.status).toBe(1);
    expect(r.out).toContain('fixtures/hang-after-result.mjs FAILED: printed its result but was still running');
  });
  it('fails a suite that exits without a result line', () => {
    const r = run(['fixtures/no-result.mjs', 'fixtures/pass.mjs']);
    expect(r.status).toBe(1);
    expect(r.out).toContain('fixtures/no-result.mjs FAILED: exited (0) without a result line');
    expect(r.out).toContain('fixtures/pass.mjs passed');
  });
  it('fails, by name, a suite that runs past its timeout', () => {
    const r = run(['fixtures/slow.mjs'], { SUITE_SECONDS: '1' });
    expect(r.status).toBe(1);
    expect(r.out).toContain('fixtures/slow.mjs FAILED: still running after 1 s');
  });
  it('rejects a suite that never calls done()', () => {
    expect(run(['fixtures/no-done.mjs']).out).toContain('never calls done()');
  });
});
