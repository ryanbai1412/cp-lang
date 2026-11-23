import * as assert from 'node:assert/strict';
import * as path from 'node:path';
import { test } from 'node:test';
import { installHint, missingCpl, RESTRICTED, resolveServer, runCpl } from '../../src/cpl';

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

test('without PATH, the installed cpl comes before a workspace build', () => {
    const built = path.posix.join('/work/cp-lang', 'target', 'debug', 'cpl');
    const find = (files: string[], cplHome?: string) =>
        resolveServer('cpl', {
            pathEnv: '/usr/bin',
            workspaceFolders: ['/work/cp-lang'],
            platform: 'linux',
            exists: file => files.includes(file),
            home: '/home/u',
            cplHome,
        });
    assert.equal(find(['/home/u/.cpl/bin/cpl', built]), '/home/u/.cpl/bin/cpl');
    assert.equal(find(['/home/u/.cargo/bin/cpl', built]), '/home/u/.cargo/bin/cpl');
    assert.equal(find(['/opt/cpl/bin/cpl', '/home/u/.cpl/bin/cpl'], '/opt/cpl'), '/opt/cpl/bin/cpl');
    assert.equal(find(['/usr/bin/cpl', '/home/u/.cpl/bin/cpl']), '/usr/bin/cpl');
    const windows = resolveServer('cpl', {
        pathEnv: '',
        workspaceFolders: [],
        platform: 'win32',
        exists: file => file === 'C:\\Users\\u\\.cpl\\bin\\cpl.exe',
        home: 'C:\\Users\\u',
    });
    assert.equal(windows, 'C:\\Users\\u\\.cpl\\bin\\cpl.exe');
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

test('a missing cpl points at the installer and where it puts cpl, not at a source build', () => {
    const unix = installHint('darwin');
    assert.match(unix, /curl -fsSL https:\/\/cp-lang\.org\/install\.sh \| sh/);
    assert.match(unix, /~\/\.cpl\/bin/);
    const windows = installHint('win32');
    assert.match(windows, /install\.ps1/);
    assert.match(windows, /%USERPROFILE%\\\.cpl\\bin/);
    for (const hint of [unix, windows]) {
        assert.doesNotMatch(hint, /cargo/);
    }
});

test('an untrusted folder is told to trust, not to install cpl', () => {
    assert.equal(missingCpl(false, 'linux'), RESTRICTED);
    assert.match(RESTRICTED, /Restricted Mode/);
    assert.match(RESTRICTED, /[Tt]rust the folder/);
    assert.match(missingCpl(true, 'linux'), /install\.sh/);
    // Restricted Mode keeps the extension, so .cp is still CP Lang, not Plain Text.
    const manifest = require('../../../package.json');
    assert.equal(manifest.capabilities.untrustedWorkspaces.supported, 'limited');
    assert.ok(manifest.capabilities.untrustedWorkspaces.restrictedConfigurations.includes('cp.server.path'));
});
