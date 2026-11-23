// What `cpl run` and `cpl stress` report, and how it is shown. No `vscode`
// import so every parser and formatter can be unit-tested directly.

/** The value of one top-level `let` or part result on a sample. */
export interface LiveValue {
    /** 0-based line the value belongs to. */
    line: number;
    name: string;
    value: string;
}

export interface RunReport {
    values: LiveValue[];
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
}

export interface StressReport {
    status: 'pass' | 'fail';
    cases?: number;
    counterexample?: Counterexample;
}

/** One diagnostic or runtime trap parsed from `cpl` stderr (`cpl_diag::render`). */
export interface CliDiagnostic {
    severity: 'error' | 'warning' | 'info';
    code?: string;
    message: string;
    file?: string;
    /** 0-based line number when a `--> file:line:col` span is present. */
    line?: number;
    /** 0-based column number when a `--> file:line:col` span is present. */
    col?: number;
    help?: string;
}

/** Inline runtime error shown on the line where `cpl run` trapped. */
export interface RuntimeErrorOverlay {
    /** 0-based line to decorate. */
    line: number;
    /** 0-based column for the Problems diagnostic, if known. */
    col?: number;
    code?: string;
    message: string;
    sample?: string;
    help?: string;
}

/** Compact inline verdict badge shown after running a sample or all samples. */
export interface SampleStatusOverlay {
    /** 0-based line to decorate. */
    line: number;
    ok: boolean;
    label: string;
    detail?: string;
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
    const values: LiveValue[] = [];
    for (const item of root.values) {
        if (!isObject(item)) {
            continue;
        }
        const at = line(item.line);
        const name = str(item.name);
        const value = str(item.value);
        if (at !== undefined && name !== undefined && value !== undefined) {
            values.push({ line: at, name, value });
        }
    }
    const report: RunReport = { values, stdout: str(root.stdout) ?? '' };
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
    if (typeof root.cases === 'number') {
        report.cases = root.cases;
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
        report.counterexample = counterexample;
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

/** The text shown after a line for its live values: `i1: [0, 3, none, 7, 5]`. */
export function liveLabel(sample: string, values: readonly LiveValue[], max: number): string {
    const shown =
        values.length === 1 ? flat(values[0].value) : values.map(v => `${v.name} = ${flat(v.value)}`).join('  ');
    return `${sample}: ${cut(shown, max)}`;
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

export interface OutputDiff {
    same: boolean;
    /** 0-based index of the first line that differs. */
    line?: number;
    expected?: string;
    actual?: string;
}

/** Compares outputs the way judges do: trailing spaces and trailing blank lines don't count. */
export function diffOutput(expected: string, actual: string): OutputDiff {
    const norm = (text: string) => {
        const lines = text.split('\n').map(l => l.trimEnd());
        while (lines.length > 0 && lines[lines.length - 1] === '') {
            lines.pop();
        }
        return lines;
    };
    const want = norm(expected);
    const got = norm(actual);
    for (let i = 0; i < Math.max(want.length, got.length); i++) {
        if (want[i] !== got[i]) {
            return { same: false, line: i, expected: want[i], actual: got[i] };
        }
    }
    return { same: true };
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
    if (counterexample.saved) {
        lines.push(`  saved as ${counterexample.saved}`);
    }
    return lines.join('\n');
}

const DIAG_HEADER = /^(error|warning|note)(?:\[([A-Za-z0-9_]+)\])?:\s*(.+)$/;
const DIAG_LOC = /^\s*-->\s+(.+):(\d+):(\d+)\s*$/;
const DIAG_HELP = /^\s*=\s*(?:help|note):\s*(.+)$/;

/**
 * Parses `cpl_diag::render` diagnostics and runtime traps from `cpl` stderr:
 *   error[E0301]: `deg` is indexed by V, but `e` is an E
 *     --> d.cp:7:17
 *     = help: index with a V
 */
export function parseCliDiagnostics(stderr: string): CliDiagnostic[] {
    const diagnostics: CliDiagnostic[] = [];
    let current: CliDiagnostic | undefined;
    for (const raw of stderr.split(/\r?\n/)) {
        const header = DIAG_HEADER.exec(raw);
        if (header) {
            const severity = header[1] === 'error' ? 'error' : header[1] === 'warning' ? 'warning' : 'info';
            current = { severity, code: header[2], message: header[3].trim() };
            diagnostics.push(current);
            continue;
        }
        if (!current) {
            continue;
        }
        const loc = DIAG_LOC.exec(raw);
        if (loc && current.line === undefined) {
            const lineNum = Number.parseInt(loc[2], 10);
            const colNum = Number.parseInt(loc[3], 10);
            current.file = loc[1];
            if (lineNum >= 1) {
                current.line = lineNum - 1;
            }
            if (colNum >= 1) {
                current.col = colNum - 1;
            }
            continue;
        }
        const help = DIAG_HELP.exec(raw);
        if (help) {
            current.help = current.help ? `${current.help}; ${help[1].trim()}` : help[1].trim();
        }
    }
    return diagnostics;
}

/** Inline label shown at the end of a line that trapped at runtime. */
export function runtimeErrorLabel(error: RuntimeErrorOverlay, max: number): string {
    const prefix = error.sample ? `${error.sample} error: ` : 'runtime error: ';
    const code = error.code ? `[${error.code}] ` : '';
    return `${prefix}${clip(code + error.message, max)}`;
}

/** Inline label shown beside a sample or file after `cpl run`. */
export function sampleStatusLabel(status: SampleStatusOverlay, max: number): string {
    const icon = status.ok ? '\u2713' : '\u2717';
    return `${icon} ${clip(status.label, max)}`;
}

/** What the trace viewer in a webview tells the editor: the line of the step it shows, or a line clicked. */
export interface TraceMessage {
    type: 'step' | 'reveal';
    /** 1-based. */
    line: number;
}

export function parseTraceMessage(message: unknown): TraceMessage | undefined {
    if (typeof message !== 'object' || message === null) {
        return undefined;
    }
    const { type, line } = message as { type?: unknown; line?: unknown };
    if ((type !== 'step' && type !== 'reveal') || typeof line !== 'number' || !Number.isInteger(line) || line < 1) {
        return undefined;
    }
    return { type, line };
}

/** What `cpl submit <file> --json` reports. */
export interface SubmitReport {
    ready: boolean;
    path?: string;
    bytes?: number;
    limit?: number;
    judge?: string;
    passed?: number;
    total?: number;
}

/** Parses the output of `cpl submit <file> --json`; undefined if it is not that. */
export function parseSubmitReport(stdout: string): SubmitReport | undefined {
    let json: unknown;
    try {
        json = JSON.parse(stdout.trim());
    } catch {
        return undefined;
    }
    if (typeof json !== 'object' || json === null || typeof (json as { ready?: unknown }).ready !== 'boolean') {
        return undefined;
    }
    const report = json as { ready: boolean; build?: Record<string, unknown>; test?: Record<string, unknown> };
    const num = (v: unknown) => (typeof v === 'number' ? v : undefined);
    const str = (v: unknown) => (typeof v === 'string' ? v : undefined);
    return {
        ready: report.ready,
        path: str(report.build?.path),
        bytes: num(report.build?.bytes),
        limit: num(report.build?.limit),
        judge: str(report.build?.judge),
        passed: num(report.test?.passed),
        total: num(report.test?.total),
    };
}

/** One line for the submit result: `ready to send: a.cpp (4.5 KB, 7% of codeforces's limit), 2/2 samples`. */
export function submitSummary(report: SubmitReport): string {
    const size =
        report.bytes !== undefined
            ? ` (${(report.bytes / 1024).toFixed(1)} KB` +
              (report.limit ? `, ${Math.round((100 * report.bytes) / report.limit)}% of ${report.judge ?? 'the judge'}'s limit` : '') +
              ')'
            : '';
    const samples = report.total !== undefined ? `, ${report.passed ?? 0}/${report.total} samples` : ', no samples';
    return `${report.ready ? 'ready to send' : 'not ready to send'}: ${report.path ?? 'the file'}${size}${samples}`;
}
