// The `view` cpl reports carry, and clipping its labels. No `vscode` import
// so it can be unit-tested directly.

/** What an editor draws for a report (`view` in spec/tools/cli.md). */

export interface View {
    panel: string;
    status?: { line: number; ok: boolean; label: string; detail?: string };
    lines?: { line: number; kind: 'error' | 'differs' | 'agrees' | 'input' | 'live'; label: string; hover?: string }[];
    diagnostic?: { line: number; col?: number; code?: string; message: string; help?: string; source?: string };
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
