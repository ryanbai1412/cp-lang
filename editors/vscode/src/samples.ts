// Finding samples and parts in a `.cp` file. No `vscode` import: these are
// plain functions over text so they can be unit-tested.

/** A sample as `cpl run --samples --list` names it, its files as paths. */
export interface Sample {
    label: string;
    input: string;
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
    command: 'cp.replayCall';
    args: unknown[];
}

/**
 * While a debug session is stopped: a lens above each fn of the solution
 * that runs its call at the current step again with the fn as edited.
 */
export function replayLenses(text: string): LensSpec[] {
    const tools = new Set(['alt', 'check', 'interactor', 'grader']);
    return parts(text)
        .filter(part => !part.decorators.some(d => tools.has(d.name)))
        .map(part => ({ line: part.fnLine, title: '$(debug-restart) Replay this call', command: 'cp.replayCall', args: [{ fn: part.name }] }));
}
