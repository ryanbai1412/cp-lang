// Runs `suite.ts` inside a real extension host with the extension loaded.
// `VSCODE_EXECUTABLE` names the editor to use (VS Code, VSCodium, ...);
// without it the current stable VS Code is downloaded. `CPL` names the
// `cpl` binary; without it the repository's own build is used.

import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { runTests } from '@vscode/test-electron';
import { resolveServer } from '../../src/cpl';

async function main(): Promise<void> {
    const extensionRoot = path.resolve(__dirname, '..', '..', '..');
    const repoRoot = path.resolve(extensionRoot, '..', '..');
    const cpl = process.env.CPL ?? resolveServer('cpl', { pathEnv: '', workspaceFolders: [repoRoot] });
    if (!cpl) {
        throw new Error('no cpl binary: run `cargo build -p cpl-cli` or set CPL');
    }
    // The tests edit files, so they get a copy of the fixture workspace.
    const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'cp-vscode-'));
    const workspace = path.join(scratch, 'workspace');
    fs.cpSync(path.join(extensionRoot, 'test', 'fixtures', 'workspace'), workspace, { recursive: true });
    try {
        await runTests({
            vscodeExecutablePath: process.env.VSCODE_EXECUTABLE,
            extensionDevelopmentPath: extensionRoot,
            extensionTestsPath: path.join(__dirname, 'suite.js'),
            extensionTestsEnv: {
                CPL_REAL: cpl,
                CPL_STUB: path.join(extensionRoot, 'test', 'fixtures', 'cpl-stub.js'),
            },
            launchArgs: [
                workspace,
                '--disable-extensions',
                '--disable-workspace-trust',
                '--user-data-dir',
                path.join(scratch, 'user-data'),
            ],
        });
        // As a desktop launcher starts it: a home with only the installer's
        // `.cpl/bin/cpl`, no setting, and a PATH without `cpl`. Not on macOS,
        // where VS Code under a fresh HOME waits on CI with no output.
        if (process.platform === 'darwin') {
            return;
        }
        const windows = process.platform === 'win32';
        const home = path.join(scratch, 'home');
        const installed = path.join(home, '.cpl', 'bin', windows ? 'cpl.exe' : 'cpl');
        fs.mkdirSync(path.dirname(installed), { recursive: true });
        fs.copyFileSync(cpl, installed);
        fs.chmodSync(installed, 0o755);
        const bare = path.join(scratch, 'bare');
        fs.cpSync(path.join(extensionRoot, 'test', 'fixtures', 'workspace'), bare, { recursive: true });
        const system = windows ? path.join(process.env.SystemRoot ?? 'C:\\Windows', 'System32') : '/usr/bin:/bin';
        await runTests({
            vscodeExecutablePath: process.env.VSCODE_EXECUTABLE,
            extensionDevelopmentPath: extensionRoot,
            extensionTestsPath: path.join(__dirname, 'installed.js'),
            extensionTestsEnv: {
                // The key Windows spells `Path`, so it is replaced, not doubled.
                [Object.keys(process.env).find(key => key.toUpperCase() === 'PATH') ?? 'PATH']: system,
                [windows ? 'USERPROFILE' : 'HOME']: home,
                CPL_HOME: '',
                CPL_INSTALLED: installed,
            },
            launchArgs: [
                bare,
                '--disable-extensions',
                '--disable-workspace-trust',
                '--user-data-dir',
                path.join(scratch, 'user-data-installed'),
                '--extensions-dir',
                path.join(scratch, 'extensions-installed'),
            ],
        });
    } finally {
        fs.rmSync(scratch, { recursive: true, force: true });
    }
}

main().catch(error => {
    console.error(error);
    process.exit(1);
});
