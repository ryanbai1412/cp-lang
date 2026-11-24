import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { covers, dependencies, hashFiles, repo, undeclared, web } from './cache.mjs';
import { stages } from './stages.mjs';

let dir, rel;
beforeEach(() => {
  dir = mkdtempSync(join(web, '.prebuild-test-'));
  rel = relative(repo, dir);
  mkdirSync(join(dir, 'sub'));
  writeFileSync(join(dir, 'a.cp'), 'a');
  writeFileSync(join(dir, 'sub/b.cp'), 'b');
  writeFileSync(join(dir, 'sub/c.md'), 'c');
});
afterEach(() => rmSync(dir, { recursive: true, force: true }));

describe('prebuild cache', () => {
  it('covers a directory, a suffix pattern, and honours exclusions in order', () => {
    expect(covers([rel], join(dir, 'sub/c.md'))).toBe(true);
    expect(covers([`${rel}/**/*.cp`], join(dir, 'sub/c.md'))).toBe(false);
    expect(covers([`${rel}/**/*.cp`], join(dir, 'sub/b.cp'))).toBe(true);
    expect(covers([rel, `!${rel}/sub`], join(dir, 'sub/b.cp'))).toBe(false);
    expect(covers([rel, `!${rel}/sub`], join(dir, 'a.cp'))).toBe(true);
  });

  it('changes its hash when a covered file changes, and only then', () => {
    const specs = [`${rel}/**/*.cp`];
    const before = hashFiles(specs);
    writeFileSync(join(dir, 'sub/c.md'), 'not an input');
    expect(hashFiles(specs)).toBe(before);
    writeFileSync(join(dir, 'sub/b.cp'), 'b2');
    expect(hashFiles(specs)).not.toBe(before);
    writeFileSync(join(dir, 'sub/new.cp'), 'n');
    const added = hashFiles(specs);
    rmSync(join(dir, 'sub/new.cp'));
    expect(hashFiles(specs)).not.toBe(added);
  });

  it('names a read outside the declared inputs, but not the stage code or the way to an input', () => {
    const stage = { name: 'facts', inputs: [`${rel}/sub`], outputs: [] };
    const reads = [dir, join(dir, 'sub/b.cp'), join(dir, 'a.cp'), join(web, 'scripts/facts.mjs'), join(repo, 'node_modules/x'), '/etc/hosts'];
    expect(undeclared(stage, reads)).toEqual([`${rel}/a.cp`]);
  });

  it('traces the files a stage reads and the paths it passes to a child process', () => {
    const trace = join(dir, 'trace');
    const tracer = pathToFileURL(fileURLToPath(new URL('./trace-reads.mjs', import.meta.url))).href;
    const code = `import fs from 'node:fs'; import { execFileSync } from 'node:child_process';
      fs.readFileSync(${JSON.stringify(join(dir, 'a.cp'))});
      execFileSync('cat', [${JSON.stringify(join(dir, 'sub/b.cp'))}]);`;
    const r = spawnSync(process.execPath, ['--import', tracer, '--input-type=module', '-e', code], { env: { ...process.env, PREBUILD_TRACE: trace } });
    expect(r.status).toBe(0);
    const seen = readFileSync(trace, 'utf8').split('\n');
    expect(seen).toContain(join(dir, 'a.cp'));
    expect(seen).toContain(join(dir, 'sub/b.cp'));
  });

  it('runs a stage after every stage whose outputs it reads', () => {
    const deps = dependencies(stages);
    const after = (name) => deps.get(name).map((s) => s.name);
    expect(after('gate')).toEqual(expect.arrayContaining(['wasm', 'gen-ref', 'lint-inputs', 'practice']));
    expect(after('lint-inputs')).toEqual(['wasm']);
    expect(after('facts')).toEqual(expect.arrayContaining(['gen-ref', 'releases']));
    expect(after('wasm')).toEqual([]);
  });

  it('fails on stages that read each other\'s outputs', () => {
    const a = { name: 'a', inputs: [`${rel}/x`], outputs: [`${rel}/y`] };
    const b = { name: 'b', inputs: [`${rel}/y/z`], outputs: [`${rel}/x`] };
    expect(() => dependencies([a, b])).toThrow('a -> b -> a');
    expect(dependencies([a, { ...b, outputs: [] }]).get('b').map((s) => s.name)).toEqual(['a']);
  });
});
