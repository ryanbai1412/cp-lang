// What `cpl run` and `cpl stress` report, and how it is shown. No `vscode`
// import so every parser and formatter can be unit-tested directly.

/** What an editor draws for a report (`view` in spec/tools/cli.md). */

export interface View {
    panel: string;
    status?: { line: number; ok: boolean; label: string; detail?: string };
    lines?: { line: number; kind: 'error' | 'differs' | 'agrees' | 'input' | 'live'; label: string; hover?: string }[];
    diagnostic?: { line: number; col?: number; code?: string; message: string; help?: string; source?: string };
}

export interface RunReport {
    view?: View;
    stdout: string;
    timeMs?: number;
    /** A runtime error (index out of range, failed assert, overflow). */
    error?: { line?: number; message: string };
}

function isObject(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function parseObject(json: string): Record<string, unknown> | undefined {
    try {
        const value: unknown = JSON.parse(json);
        return isObject(value) ? value : undefined;
    } catch {
        return undefined;
    }
}

function str(value: unknown): string | undefined {
    return typeof value === 'string' ? value : undefined;
}

/** A 1-based line from the compiler as a 0-based editor line. */
function line(value: unknown): number | undefined {
    return typeof value === 'number' && Number.isInteger(value) && value >= 1 ? value - 1 : undefined;
}

/** Parses the output of `cpl run <file> --format json`. Undefined when it is not that. */
export function parseRunReport(json: string): RunReport | undefined {
    const root = parseObject(json);
    if (!root || !Array.isArray(root.values)) {
        return undefined;
    }
    const report: RunReport = { stdout: str(root.stdout) ?? '' };
    if (isObject(root.view)) {
        report.view = root.view as unknown as View;
    }
    if (typeof root.time_ms === 'number') {
        report.timeMs = root.time_ms;
    }
    if (isObject(root.error) && str(root.error.message) !== undefined) {
        report.error = { line: line(root.error.line), message: str(root.error.message)! };
    }
    return report;
}

function flat(text: string): string {
    return text.replace(/\s+/g, ' ').trim();
}

function cut(text: string, max: number): string {
    return text.length <= max ? text : text.slice(0, Math.max(1, max - 1)).trimEnd() + '\u2026';
}

/** Puts `text` on one line of at most `max` characters, ending in an ellipsis when cut. */
export function clip(text: string, max: number): string {
    return cut(flat(text), max);
}

/** The `view` of a `cpl ... --json` report; undefined when the output is not one. */
export function viewOf(stdout: string): View | undefined {
    try {
        return JSON.parse(stdout).view ?? undefined;
    } catch {
        return undefined;
    }
}
