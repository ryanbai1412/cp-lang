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
    } finally {
        fs.rmSync(scratch, { recursive: true, force: true });
    }
}

main().catch(error => {
    console.error(error);
    process.exit(1);
});
