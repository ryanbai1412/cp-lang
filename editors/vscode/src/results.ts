// What `cpl run` and `cpl stress` report, and how it is shown. No `vscode`
// import so every parser and formatter can be unit-tested directly.

/** What an editor draws for a report (`view` in spec/tools/cli.md), strongest kind first. */
export const VIEW_KINDS = ['error', 'differs', 'agrees', 'input', 'live'] as const;

export interface View {
    panel: string;
    status?: { line: number; ok: boolean; label: string; detail?: string };
    lines?: { line: number; kind: (typeof VIEW_KINDS)[number]; label: string; hover?: string }[];
    diagnostic?: { line: number; col?: number; code?: string; message: string; help?: string; source?: string };
}

export interface RunReport {
    view?: View;
    stdout: string;
    timeMs?: number;
    /** A runtime error (index out of range, failed assert, overflow). */
    error?: { line?: number; message: string };
}

/** One part on the counterexample: what the fast code and the brute returned. */
export interface StageDiff {
    name: string;
    line?: number;
    fast: string;
    brute: string;
    differs: boolean;
}

/** Where the solution and its alt first part ways on the counterexample. */
export interface Divergence {
    /** A part call, `tri(k: 1)`, or a value, `s`. */
    name: string;
    /** Which call of the part, or which value, from 1. */
    nth: number;
    call: boolean;
    /** 0-based lines and values of each side; undefined when that side never gets there. */
    fast?: { line?: number; value: string };
    brute?: { line?: number; value: string };
    /** The solution's last steps inside the wrong call. */
    inside: { line?: number; locals: string }[];
}

export interface Counterexample {
    /** The shrunk input, as `cpl stress` prints it: `n = 2  a = [7, 7]`. */
    input: string;
    caseNumber?: number;
    seed?: number;
    verdict?: string;
    expected?: string;
    got?: string;
    rawInput?: string;
    stages: StageDiff[];
    /** Where the case was saved, such as `i6` or `d.tests/stress-17.in`. */
    saved?: string;
    divergence?: Divergence;
}

export interface StressReport {
    status: 'pass' | 'fail';
    cases?: number;
    counterexample?: Counterexample;
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

/**
 * Parses the output of `cpl stress <file> --format json`.
 * Accepts both `"status": "ok"` (emitted by `cpl-stress`) and `"status": "pass"`.
 */
export function parseStressReport(json: string): StressReport | undefined {
    const root = parseObject(json);
    if (!root || (root.status !== 'pass' && root.status !== 'ok' && root.status !== 'fail')) {
        return undefined;
    }
    const status: 'pass' | 'fail' = root.status === 'fail' ? 'fail' : 'pass';
    const report: StressReport = { status };
    if (typeof root.case_count === 'number') {
        report.cases = root.case_count;
    }
    const found = root.counterexample;
    if (status === 'fail' && isObject(found)) {
        const stages: StageDiff[] = [];
        if (Array.isArray(found.stages)) {
            for (const item of found.stages) {
                if (!isObject(item)) {
                    continue;
                }
                const name = str(item.name);
                const fast = str(item.fast);
                const brute = str(item.brute);
                if (name !== undefined && fast !== undefined && brute !== undefined) {
                    stages.push({ name, line: line(item.line), fast, brute, differs: fast !== brute });
                }
            }
        }
        const counterexample: Counterexample = {
            input: str(found.input) ?? '',
            caseNumber: typeof found.case === 'number' ? found.case : undefined,
            stages,
            saved: str(found.saved),
        };
        if (typeof found.seed === 'number') {
            counterexample.seed = found.seed;
        }
        if (str(found.verdict) !== undefined) {
            counterexample.verdict = str(found.verdict);
        }
        if (str(found.expected) !== undefined) {
            counterexample.expected = str(found.expected);
        }
        if (str(found.got) !== undefined) {
            counterexample.got = str(found.got);
        }
        if (str(found.raw_input) !== undefined) {
            counterexample.rawInput = str(found.raw_input);
        }
        const divergence = isObject(found.explain) ? parseDivergence(found.explain.divergence) : undefined;
        if (divergence) {
            counterexample.divergence = divergence;
        }
        report.counterexample = counterexample;
    }
    return report;
}

function parseDivergence(value: unknown): Divergence | undefined {
    if (!isObject(value) || str(value.name) === undefined || typeof value.nth !== 'number') {
        return undefined;
    }
    const side = (at: unknown) =>
        isObject(at) && str(at.value) !== undefined ? { line: line(at.line), value: str(at.value)! } : undefined;
    const inside = Array.isArray(value.inside)
        ? value.inside.filter(isObject).map(step => ({ line: line(step.line), locals: str(step.locals) ?? '' }))
        : [];
    return {
        name: str(value.name)!,
        nth: value.nth,
        // Older `cpl` builds name only part calls with arguments.
        call: typeof value.call === 'boolean' ? value.call : str(value.name)!.includes('('),
        fast: side(value.fast),
        brute: side(value.brute),
        inside,
    };
}

/** `1st`, `2nd`, `3rd`, `11th`. */
function ordinal(n: number): string {
    const suffix = n % 100 >= 11 && n % 100 <= 13 ? 'th' : (['th', 'st', 'nd', 'rd'][n % 10] ?? 'th');
    return `${n}${suffix}`;
}

/** The headline `cpl stress` prints for a divergence. */
export function divergenceTitle(d: Divergence): string {
    const f = d.name.split('(')[0];
    return d.call
        ? `first wrong call: \`${d.name}\`, the ${ordinal(d.nth)} call of \`${f}\``
        : `first differing value: \`${d.name}\`, its ${ordinal(d.nth)} value`;
}

/** The text shown after the solution's line of the divergence. */
export function divergenceLabel(d: Divergence, max: number): string {
    const fast = d.fast ? clip(d.fast.value, max) : 'nothing';
    const brute = d.brute ? clip(d.brute.value, max) : 'nothing';
    const what = d.call ? 'first wrong call' : 'first differing value';
    return `${d.name} = ${fast}  \u2260  brute ${brute}  \u2190 ${what}`;
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

/** The text shown after the line of a part on the counterexample. */
export function stageLabel(stage: StageDiff, max: number): string {
    return stage.differs
        ? `fast ${clip(stage.fast, max)}  \u2260  brute ${clip(stage.brute, max)}  \u2190 first difference`
        : `fast = brute = ${clip(stage.fast, max)}`;
}

/** Summary decoration for line 0 when a counterexample is active. */
export function counterexampleSummary(counterexample: Counterexample, max: number): string {
    const input = clip(counterexample.input || counterexample.rawInput || '(empty input)', max);
    if (counterexample.stages.length === 0 && (counterexample.got !== undefined || counterexample.expected !== undefined)) {
        const got = clip(counterexample.got ?? '', Math.max(12, Math.floor(max / 2)));
        const expected = clip(counterexample.expected ?? '', Math.max(12, Math.floor(max / 2)));
        if (counterexample.expected !== undefined && counterexample.got !== undefined) {
            return `counterexample: ${input}  (fast ${got} \u2260 brute ${expected})`;
        }
        if (counterexample.verdict) {
            return `counterexample (${counterexample.verdict}): ${input}`;
        }
    }
    return `counterexample: ${input}`;
}

/** The first part where the fast code and the brute disagree. */
export function firstDifference(counterexample: Counterexample): StageDiff | undefined {
    return counterexample.stages.find(stage => stage.differs);
}

/** The counterexample as the output panel shows it, the same layout as `cpl stress`. */
export function renderCounterexample(counterexample: Counterexample): string {
    const lines: string[] = [];
    const at = counterexample.caseNumber !== undefined ? ` at case ${counterexample.caseNumber}` : '';
    const seed = counterexample.seed !== undefined ? ` (seed ${counterexample.seed})` : '';
    const verdict = counterexample.verdict ? ` [${counterexample.verdict}]` : '';
    lines.push(`FAIL${at}${seed}${verdict}, shrunk to:`);
    lines.push(`  ${counterexample.input || flat(counterexample.rawInput ?? '')}`);
    if (counterexample.stages.length > 0) {
        const width = (pick: (stage: StageDiff) => string, title: string) =>
            Math.max(title.length, ...counterexample.stages.map(s => pick(s).length)) + 2;
        const nameWidth = width(s => s.name, 'part');
        const fastWidth = width(s => s.fast, 'fast');
        lines.push(`  ${'part'.padEnd(nameWidth)}${'fast'.padEnd(fastWidth)}brute`);
        let marked = false;
        for (const stage of counterexample.stages) {
            const mark = stage.differs && !marked ? '   <- first difference' : '';
            marked = marked || stage.differs;
            lines.push(`  ${stage.name.padEnd(nameWidth)}${stage.fast.padEnd(fastWidth)}${stage.brute}${mark}`);
        }
    } else if (counterexample.expected !== undefined || counterexample.got !== undefined) {
        if (counterexample.got !== undefined) {
            lines.push(`  fast:  ${flat(counterexample.got)}`);
        }
        if (counterexample.expected !== undefined) {
            lines.push(`  brute: ${flat(counterexample.expected)}`);
        }
    }
    const d = counterexample.divergence;
    if (d) {
        lines.push(`  ${divergenceTitle(d)}`);
        for (const [who, side] of [['solution', d.fast], ['brute', d.brute]] as const) {
            lines.push(
                side
                    ? `    ${who.padEnd(8)} ${d.name} = ${side.value}${side.line !== undefined ? ` at line ${side.line + 1}` : ''}`
                    : `    ${who.padEnd(8)} never gives \`${d.name}\` a ${ordinal(d.nth)} value`,
            );
        }
        if (d.inside.length > 0) {
            lines.push("    the solution's last steps in it:");
            for (const step of d.inside) {
                lines.push(`      line ${step.line !== undefined ? step.line + 1 : '?'}: ${step.locals}`);
            }
        }
    }
    if (counterexample.saved) {
        lines.push(`  saved as ${counterexample.saved}`);
    }
    return lines.join('\n');
}

/** The `view` of a `cpl ... --json` report; undefined when the output is not one. */
export function viewOf(stdout: string): View | undefined {
    try {
        return JSON.parse(stdout).view ?? undefined;
    } catch {
        return undefined;
    }
}

/** One case of `cpl test --format json`. */
export interface TestCase {
    name: string;
    /** `ok`, `WA`, `RE`, `TLE`, `MLE`, ...; undefined when the case had no answer to judge by. */
    verdict?: string;
    ms: number;
    expected?: string;
    output: string;
    why?: string;
    explained?: string;
}

/** Parses the output of `cpl test <problem> --format json`. Undefined when it is not that. */
export function parseTestReport(json: string): TestCase[] | undefined {
    const root = parseObject(json);
    if (!root || !Array.isArray(root.cases)) {
        return undefined;
    }
    const cases: TestCase[] = [];
    for (const item of root.cases) {
        const name = isObject(item) ? str(item.name) : undefined;
        if (!isObject(item) || name === undefined) {
            continue;
        }
        cases.push({
            name,
            verdict: str(item.verdict),
            ms: typeof item.time_ms === 'number' ? item.time_ms : 0,
            expected: str(item.expected),
            output: str(item.output) ?? '',
            why: str(item.why),
            explained: str(item.explained),
        });
    }
    return cases;
}
