// Runs in an extension host started the way a desktop launcher starts VS
// Code: no `cpl` on PATH and no setting, only the installer's `~/.cpl/bin`.

import * as assert from 'node:assert/strict';
import * as vscode from 'vscode';
import { serverBinary } from '../../src/client';

export async function run(): Promise<void> {
    assert.equal(serverBinary(), process.env.CPL_INSTALLED);
    const uri = vscode.Uri.joinPath(vscode.workspace.workspaceFolders![0].uri, 'broken.cp');
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    const deadline = Date.now() + 60000;
    while (vscode.languages.getDiagnostics(document.uri).length === 0) {
        if (Date.now() > deadline) {
            throw new Error(`no diagnostics from ${process.env.CPL_INSTALLED} with PATH=${process.env.PATH}`);
        }
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    console.log(`ok - found the installed cpl without PATH: ${serverBinary()}`);
}
