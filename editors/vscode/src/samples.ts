// Finding samples and parts in a `.cp` file. No `vscode` import: these are
// plain functions over text so they can be unit-tested.

/** A sample to run the program on. */
export interface Sample {
    /** How the sample is named in lenses and output: `i1`, `1`, `sample 2`. */
    label: string;
    /** Inline input text, or the path of the input file relative to the source's folder. */
    input: { text: string } | { file: string };
    expected?: { text: string } | { file: string };
}

export interface Decorator {
    name: string;
    /** The text between the parentheses, or undefined when there are none. */
    args?: string;
}

/** A top-level fn with its decorators. */
export interface Part {
    name: string;
    /** Line of the first decorator, or of the `fn` when there is none (0-based). */
    line: number;
    fnLine: number;
    decorators: Decorator[];
}

function byNumber(a: string, b: string): number {
    return parseInt(a.replace(/\D/g, ''), 10) - parseInt(b.replace(/\D/g, ''), 10) || a.localeCompare(b);
}

/**
 * The samples kept as files beside `<stem>.cp` (`plan/40-tooling.md` section 1):
 * scratch inputs `i1`, `i2`, ... with `o1` as the expected output, then
 * `<stem>.tests/N.in` and `<stem>.samples/N.in` with `N.out`.
 */
export function fileSamples(
    stem: string,
    folder: readonly string[],
    tests: readonly string[],
    samples: readonly string[],
): Sample[] {
    const out: Sample[] = [];
    const scratch = folder.filter(name => /^i\d+$/.test(name)).sort(byNumber);
    for (const name of scratch) {
        const expected = 'o' + name.slice(1);
        out.push({
            label: name,
            input: { file: name },
            expected: folder.includes(expected) ? { file: expected } : undefined,
        });
    }
    for (const [dir, names] of [[`${stem}.tests`, tests], [`${stem}.samples`, samples]] as const) {
        const inputs = names.filter(name => name.endsWith('.in')).sort(byNumber);
        for (const name of inputs) {
            const base = name.slice(0, -'.in'.length);
            out.push({
                label: base,
                input: { file: `${dir}/${name}` },
                expected: names.includes(`${base}.out`) ? { file: `${dir}/${base}.out` } : undefined,
            });
        }
    }
    return out;
}

const DECORATOR = /@([A-Za-z_][A-Za-z0-9_]*)(?:\(((?:[^()]|\([^()]*\))*)\))?/g;
const FN = /^(?:pub\s+)?((?:@[A-Za-z_][A-Za-z0-9_]*(?:\((?:[^()]|\([^()]*\))*\))?\s+)*)fn\s+([A-Za-z_][A-Za-z0-9_]*)/;
const DECORATOR_LINE = /^(?:@[A-Za-z_][A-Za-z0-9_]*(?:\((?:[^()]|\([^()]*\))*\))?\s*)+(?:\/\/.*)?$/;

function decoratorsIn(text: string): Decorator[] {
    return [...text.matchAll(DECORATOR)].map(m => ({ name: m[1], args: m[2] }));
}

/** The file's top-level fns (no indentation), each with the decorators above and before it. */
export function parts(text: string): Part[] {
    const out: Part[] = [];
    const lines = text.split('\n');
    let pending: Decorator[] = [];
    let pendingLine = -1;
    lines.forEach((line, index) => {
        const fn = FN.exec(line);
        if (fn) {
            const decorators = [...pending, ...decoratorsIn(fn[1])];
            out.push({
                name: fn[2],
                line: pending.length > 0 ? pendingLine : index,
                fnLine: index,
                decorators,
            });
            pending = [];
        } else if (DECORATOR_LINE.test(line)) {
            if (pending.length === 0) {
                pendingLine = index;
            }
            pending.push(...decoratorsIn(line.replace(/\/\/.*$/, '')));
        } else if (line.trim() !== '' && !line.trim().startsWith('//')) {
            pending = [];
        }
    });
    return out;
}

/** One lens: a title, the command it runs and that command's arguments after the document URI. */
export interface LensSpec {
    line: number;
    title: string;
    command: 'cp.runSample' | 'cp.runAllSamples' | 'cp.stress' | 'cp.trace';
    args: unknown[];
}

/**
 * The lenses for a file (`plan/60-editor.md` section 6.4): a Run row above
 * the file, one above each inline sample, and Stress / Trace lenses above
 * `@brute`, `@checker` and `@interactor` functions.
 */
export function planLenses(text: string, samples: readonly Sample[]): LensSpec[] {
    const out: LensSpec[] = [];
    const fns = parts(text);
    const bruted = fns.filter(p => p.decorators.some(d => d.name === 'brute'));
    const checkers = fns.filter(p => p.decorators.some(d => d.name === 'checker'));
    const interactors = fns.filter(p => p.decorators.some(d => d.name === 'interactor'));
    const hasStressTarget = bruted.length > 0 || checkers.length > 0 || interactors.length > 0;
    const first = samples[0];
    out.push({
        line: 0,
        title: first ? `$(play) Run on ${first.label}` : '$(play) Run',
        command: 'cp.runSample',
        args: first ? [first] : [],
    });
    if (samples.length > 1) {
        out.push({ line: 0, title: `Run on all ${samples.length} samples`, command: 'cp.runAllSamples', args: [] });
    }
    if (hasStressTarget) {
        out.push({ line: 0, title: 'Stress', command: 'cp.stress', args: [] });
    }
    out.push({ line: 0, title: 'Trace', command: 'cp.trace', args: first ? [first] : [] });
    for (const part of bruted) {
        const target = part.decorators.find(d => d.name === 'brute')?.args?.trim();
        if (target === 'main') {
            out.push({
                line: part.line,
                title: `Stress main against ${part.name}`,
                command: 'cp.stress',
                args: [],
            });
            out.push({ line: part.line, title: 'Trace', command: 'cp.trace', args: [first] });
        } else {
            out.push({
                line: part.line,
                title: target ? `Stress against ${target}` : 'Stress',
                command: 'cp.stress',
                args: [part.name],
            });
            out.push({ line: part.line, title: 'Trace', command: 'cp.trace', args: [first, part.name] });
        }
    }
    for (const checker of checkers) {
        out.push({
            line: checker.line,
            title: `Stress with checker ${checker.name}`,
            command: 'cp.stress',
            args: [],
        });
    }
    for (const interactor of interactors) {
        out.push({
            line: interactor.line,
            title: `Stress with interactor ${interactor.name}`,
            command: 'cp.stress',
            args: [],
        });
    }
    return out;
}
