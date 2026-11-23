// Finding and running the `cpl` binary. No `vscode` import.

import { spawn } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';

export interface ResolveOptions {
    /** The value of `PATH`. */
    pathEnv: string;
    workspaceFolders: readonly string[];
    platform?: NodeJS.Platform;
    exists?: (file: string) => boolean;
    /** When `file` was last written, in ms. */
    modified?: (file: string) => number;
}

function modifiedAt(file: string): number {
    try {
        return fs.statSync(file).mtimeMs;
    } catch {
        return 0;
    }
}

function isFile(file: string): boolean {
    try {
        return fs.statSync(file).isFile();
    } catch {
        return false;
    }
}

/**
 * The binary to run for the setting `cp.server.path`. A path is taken as
 * written (relative ones against each workspace folder). A bare name is
 * looked up on `PATH`, then in each workspace folder's `target/release` and
 * `target/debug`, the newer build first, so the extension works inside the
 * cp-lang repository without installing anything and never runs a stale build.
 */
export function resolveServer(configured: string, options: ResolveOptions): string | undefined {
    const exists = options.exists ?? isFile;
    const windows = (options.platform ?? process.platform) === 'win32';
    const paths = windows ? path.win32 : path.posix;
    const names = windows && !/\.exe$/i.test(configured) ? [configured, configured + '.exe'] : [configured];
    const firstIn = (dirs: readonly string[]) => {
        for (const dir of dirs) {
            for (const name of names) {
                const candidate = paths.join(dir, name);
                if (exists(candidate)) {
                    return candidate;
                }
            }
        }
        return undefined;
    };
    if (configured.trim() === '') {
        return undefined;
    }
    if (paths.isAbsolute(configured)) {
        return names.find(exists);
    }
    if (/[\\/]/.test(configured)) {
        return firstIn(options.workspaceFolders);
    }
    const onPath = options.pathEnv.split(windows ? ';' : ':').filter(dir => dir !== '');
    const modified = options.modified ?? modifiedAt;
    const built = options.workspaceFolders.flatMap(folder =>
        [paths.join(folder, 'target', 'release'), paths.join(folder, 'target', 'debug')]
            .map(dir => ({ dir, at: Math.max(0, ...names.map(name => modified(paths.join(dir, name)))) }))
            .sort((a, b) => b.at - a.at)
            .map(({ dir }) => dir),
    );
    return firstIn(onPath) ?? firstIn(built);
}

export interface CplResult {
    /** The exit status, or null when the process was killed or never started. */
    code: number | null;
    stdout: string;
    stderr: string;
    timedOut: boolean;
    cancelled?: boolean;
    timeMs: number;
    /** Set when the process could not be started. */
    spawnError?: string;
}

/** `cpl`'s exit status when compilation failed with source diagnostics. */
export const EXIT_ERRORS = 1;
/** `cpl`'s exit status when a run/stress/test failed at runtime (trap or wrong answer). */
export const EXIT_FAILED = 3;
/** Legacy alias: `cpl`'s exit status when a command's back end is not built yet. */
export const EXIT_NOT_BUILT = 3;

export interface RunOptions {
    cwd?: string;
    stdin?: string;
    timeoutMs?: number;
    signal?: AbortSignal;
}

/** Runs `cpl` to completion and collects what it printed. Never rejects. */
export function runCpl(binary: string, args: readonly string[], options: RunOptions = {}): Promise<CplResult> {
    return new Promise(resolve => {
        const started = Date.now();
        const result: CplResult = { code: null, stdout: '', stderr: '', timedOut: false, timeMs: 0 };
        if (options.signal?.aborted) {
            result.cancelled = true;
            resolve(result);
            return;
        }
        let done = false;
        const onAbort = () => {
            result.cancelled = true;
            child.kill();
        };
        const cleanup = () => {
            clearTimeout(timer);
            options.signal?.removeEventListener('abort', onAbort);
        };
        const finish = () => {
            if (!done) {
                done = true;
                cleanup();
                result.timeMs = Date.now() - started;
                resolve(result);
            }
        };
        const child = spawn(binary, [...args], { cwd: options.cwd, stdio: 'pipe' });
        const timer = options.timeoutMs
            ? setTimeout(() => {
                  result.timedOut = true;
                  child.kill();
              }, options.timeoutMs)
            : undefined;
        options.signal?.addEventListener('abort', onAbort, { once: true });
        child.stdout.setEncoding('utf8').on('data', (chunk: string) => (result.stdout += chunk));
        child.stderr.setEncoding('utf8').on('data', (chunk: string) => (result.stderr += chunk));
        child.on('error', error => {
            result.spawnError = error.message;
            finish();
        });
        child.on('close', code => {
            result.code = code;
            finish();
        });
        // The program may exit without reading its input.
        child.stdin.on('error', () => undefined);
        child.stdin.end(options.stdin ?? '');
    });
}
