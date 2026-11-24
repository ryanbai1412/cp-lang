// The Testing panel: each problem with a `<stem>.tests/` folder, its cases
// as children, run through `cpl test --format json`. A wrong answer opens as
// a diff of the expected and the actual output.

import * as path from 'node:path';
import * as vscode from 'vscode';
import { serverBinary } from './client';
import { missingCpl, runCpl } from './cpl';

export type Outcome = 'passed' | 'failed' | 'errored' | 'skipped';

export class TestPanel implements vscode.Disposable {
    readonly controller = vscode.tests.createTestController('cp', 'CP Lang');
    private readonly subscriptions: vscode.Disposable[] = [];

    constructor() {
        this.controller.resolveHandler = item => (item ? this.resolve(item) : this.discover());
        this.controller.createRunProfile('Run', vscode.TestRunProfileKind.Run, (request, token) =>
            this.run(request, token).then(() => undefined),
        );
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.tests/*.{in,out,ans}');
        const changed = (uri: vscode.Uri) => {
            const source = sourceOf(uri);
            const item = this.controller.items.get(source.toString());
            void (item ? this.resolve(item) : this.add(source));
        };
        this.subscriptions.push(this.controller, watcher, watcher.onDidCreate(changed), watcher.onDidDelete(changed));
    }

    dispose(): void {
        this.subscriptions.forEach(s => s.dispose());
    }

    /** Every problem in the workspace that has cases. */
    async discover(): Promise<void> {
        const inputs = await vscode.workspace.findFiles('**/*.tests/*.in', '**/node_modules/**');
        const sources = new Map(inputs.map(input => [sourceOf(input).toString(), sourceOf(input)]));
        await Promise.all([...sources.values()].map(source => this.add(source)));
    }

    private async add(source: vscode.Uri): Promise<void> {
        try {
            await vscode.workspace.fs.stat(source);
        } catch {
            return;
        }
        let item = this.controller.items.get(source.toString());
        if (!item) {
            item = this.controller.createTestItem(source.toString(), path.basename(source.fsPath), source);
            item.canResolveChildren = true;
            this.controller.items.add(item);
        }
        await this.resolve(item);
    }

    /** A problem's cases, in `cpl test`'s order, as `cpl test --list` gives them. */
    private async resolve(item: vscode.TestItem): Promise<void> {
        const source = item.uri!.fsPath;
        const binary = serverBinary();
        const listed = binary
            ? await runCpl(binary, ['test', source, '--list', '--format', 'json'], { cwd: path.dirname(source), timeoutMs: 10000 })
            : undefined;
        const cases = (listed?.code === 0 ? JSON.parse(listed.stdout) : { cases: [] }).cases as { name: string; input: string }[];
        if (cases.length === 0) {
            this.controller.items.delete(item.id);
            return;
        }
        item.children.replace(
            cases.map(c =>
                this.controller.createTestItem(`${item.id}#${c.name}`, c.name, vscode.Uri.file(path.resolve(path.dirname(source), c.input))),
            ),
        );
    }

    /** Runs what `request` names (everything when it names nothing) and says how each case came out. */
    async run(request: vscode.TestRunRequest, token?: vscode.CancellationToken): Promise<Map<string, Outcome>> {
        const outcomes = new Map<string, Outcome>();
        const run = this.controller.createTestRun(request);
        const roots: vscode.TestItem[] = [];
        this.controller.items.forEach(item => roots.push(item));
        const asked = request.include ?? roots;
        const excluded = new Set((request.exclude ?? []).map(item => item.id));
        // One `cpl test` per problem: a whole problem, or each case asked for on its own.
        const jobs = new Map<vscode.TestItem, vscode.TestItem[] | undefined>();
        for (const item of asked) {
            if (excluded.has(item.id)) {
                continue;
            }
            const problem = item.parent ?? item;
            if (item.parent) {
                const picked = jobs.get(problem);
                if (!jobs.has(problem) || picked) {
                    jobs.set(problem, [...(picked ?? []), item]);
                }
            } else {
                jobs.set(problem, undefined);
            }
        }
        const binary = serverBinary();
        for (const [problem, picked] of jobs) {
            if (token?.isCancellationRequested) {
                break;
            }
            const items: vscode.TestItem[] = [];
            problem.children.forEach(child => {
                if (!excluded.has(child.id) && (!picked || picked.includes(child))) {
                    items.push(child);
                }
            });
            items.forEach(item => run.started(item));
            const record = (item: vscode.TestItem, outcome: Outcome, message?: vscode.TestMessage, ms?: number) => {
                outcomes.set(item.id, outcome);
                if (outcome === 'passed') {
                    run.passed(item, ms);
                } else if (outcome === 'skipped') {
                    run.skipped(item);
                } else {
                    run[outcome](item, message ?? new vscode.TestMessage(outcome), ms);
                }
            };
            if (!binary) {
                const message = new vscode.TestMessage(missingCpl(vscode.workspace.isTrusted));
                items.forEach(item => record(item, 'errored', message));
                continue;
            }
            const source = problem.uri!.fsPath;
            const only = picked && picked.length === 1 ? ['--case', picked[0].label] : [];
            const timeoutMs = vscode.workspace.getConfiguration('cp').get<number>('run.timeoutMs', 10000);
            const abort = new AbortController();
            const cancel = token?.onCancellationRequested(() => abort.abort());
            const result = await runCpl(binary, ['test', source, '--format', 'json', ...only], {
                cwd: path.dirname(source),
                timeoutMs: timeoutMs * Math.max(items.length, 1),
                signal: abort.signal,
            });
            cancel?.dispose();
            const cases = casesOf(result.stdout);
            if (!cases) {
                // Errors in the source, or `cpl` itself failed: every case says why.
                const why = (result.stderr || result.spawnError || result.stdout).trim();
                const message = new vscode.TestMessage(result.cancelled ? 'cancelled' : why || `cpl test exited ${result.code}`);
                items.forEach(item => record(item, result.cancelled ? 'skipped' : 'errored', message));
                continue;
            }
            const byName = new Map(cases.map(c => [c.name, c]));
            for (const item of items) {
                const found = byName.get(item.label);
                if (!found) {
                    record(item, 'skipped');
                    continue;
                }
                if (found.output) {
                    run.appendOutput(`${item.label}: ${found.output.replace(/\r?\n/g, '\r\n')}`, undefined, item);
                }
                record(item, found.outcome, message(found, item), found.time_ms);
            }
        }
        run.end();
        return outcomes;
    }
}

/** One case of `cpl test --format json`, as the CLI judged it. */
interface TestCase {
    name: string;
    outcome: Outcome;
    message: string | null;
    expected: string | null;
    output: string;
    time_ms: number;
}

function casesOf(stdout: string): TestCase[] | undefined {
    try {
        const cases = JSON.parse(stdout).cases;
        return Array.isArray(cases) ? cases : undefined;
    } catch {
        return undefined;
    }
}

/** The message VS Code shows for a case that did not pass: a diff for a wrong answer. */
function message(found: TestCase, item: vscode.TestItem): vscode.TestMessage | undefined {
    if (found.message === null) {
        return undefined;
    }
    const message =
        found.outcome === 'failed' && found.expected !== null
            ? vscode.TestMessage.diff(found.message, found.expected, found.output)
            : new vscode.TestMessage(found.message);
    if (item.uri) {
        message.location = new vscode.Location(item.uri, new vscode.Position(0, 0));
    }
    return message;
}

/** The `<stem>.cp` a `<stem>.tests/N.in` belongs to. */
function sourceOf(caseFile: vscode.Uri): vscode.Uri {
    const dir = path.dirname(caseFile.fsPath);
    return vscode.Uri.file(dir.replace(/\.tests$/, '.cp'));
}
