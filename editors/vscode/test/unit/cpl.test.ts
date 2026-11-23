import * as assert from 'node:assert/strict';
import * as path from 'node:path';
import { test } from 'node:test';
import { resolveServer, runCpl } from '../../src/cpl';

function resolve(configured: string, files: string[], platform: NodeJS.Platform = 'linux'): string | undefined {
    return resolveServer(configured, {
        pathEnv: platform === 'win32' ? 'C:\\bin;C:\\tools' : '/usr/bin:/opt/cp/bin',
        workspaceFolders: ['/work/cp-lang'],
        platform,
        exists: file => files.includes(file),
    });
}

test('a bare name is found on PATH before the workspace build', () => {
    const built = path.posix.join('/work/cp-lang', 'target', 'debug', 'cpl');
    const release = path.posix.join('/work/cp-lang', 'target', 'release', 'cpl');
    assert.equal(resolve('cpl', ['/opt/cp/bin/cpl', built]), '/opt/cp/bin/cpl');
    assert.equal(resolve('cpl', [built, release]), release);
    assert.equal(resolve('cpl', [built]), built);
    assert.equal(resolve('cpl', []), undefined);
    assert.equal(resolve('', ['/usr/bin/cpl']), undefined);
});

test('the newer of the release and debug builds wins', () => {
    const debug = path.posix.join('/work/cp-lang', 'target', 'debug', 'cpl');
    const release = path.posix.join('/work/cp-lang', 'target', 'release', 'cpl');
    const newest = (newer: string) =>
        resolveServer('cpl', {
            pathEnv: '',
            workspaceFolders: ['/work/cp-lang'],
            platform: 'linux',
            exists: file => file === debug || file === release,
            modified: file => (file === newer ? 2 : 1),
        });
    assert.equal(newest(debug), debug);
    assert.equal(newest(release), release);
});

test('a path is taken as written', () => {
    assert.equal(resolve('/abs/cpl', ['/abs/cpl']), '/abs/cpl');
    assert.equal(resolve('/abs/cpl', ['/usr/bin/cpl']), undefined);
    const relative = path.posix.join('/work/cp-lang', 'tools/cpl');
    assert.equal(resolve('tools/cpl', [relative]), relative);
});

test('runCpl collects output, feeds stdin and reports the exit status', async () => {
    const echo = 'process.stdin.on("data", d => process.stdout.write(String(d).toUpperCase())); process.stdin.on("end", () => { console.error("warn"); process.exit(3); })';
    const result = await runCpl(process.execPath, ['-e', echo], { stdin: 'abc\n' });
    assert.equal(result.stdout, 'ABC\n');
    assert.equal(result.stderr, 'warn\n');
    assert.equal(result.code, 3);
    assert.equal(result.timedOut, false);
});

test('runCpl stops a run that takes too long, honours AbortSignal, and reports a missing binary', async () => {
    const slow = await runCpl(process.execPath, ['-e', 'setTimeout(() => {}, 60000)'], { timeoutMs: 200 });
    assert.equal(slow.timedOut, true);
    assert.equal(slow.code, null);
    const controller = new AbortController();
    setTimeout(() => controller.abort(), 50);
    const cancelled = await runCpl(process.execPath, ['-e', 'setTimeout(() => {}, 60000)'], {
        signal: controller.signal,
    });
    assert.equal(cancelled.cancelled, true);
    assert.equal(cancelled.timedOut, false);
    const missing = await runCpl('/nonexistent/cpl', ['lsp']);
    assert.ok(missing.spawnError, 'expected a spawn error');
});
