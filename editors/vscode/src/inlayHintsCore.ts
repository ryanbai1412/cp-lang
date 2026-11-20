// Pure inlay-hint computation for CP Lang v2.2. No `vscode` import so unit
// tests can run directly under Node.

import { PRELUDE_TYPES, STDLIB_TYPES } from './languageData';

export interface HintSpec {
    /** 0-based line number. */
    line: number;
    /** 0-based character offset right after the identifier. */
    character: number;
    label: string;
    tooltip: string;
}

const KNOWN_TYPES = new Set<string>([...PRELUDE_TYPES, ...STDLIB_TYPES]);

const CONSTRUCTOR_FUNCS: Record<string, string> = {
    compress: 'Compressed',
    Sieve: 'Sieve',
    RollingHash: 'RollingHash',
    SuffixArray: 'SuffixArray',
    Rng: 'Rng',
};

function stripComment(line: string): string {
    let inDouble = false;
    let inSingle = false;
    for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (ch === '\\' && (inDouble || inSingle)) {
            i += 1;
            continue;
        }
        if (ch === '"' && !inSingle) {
            inDouble = !inDouble;
        } else if (ch === "'" && !inDouble) {
            // Only treat `'x'` or `'\n'` as char literals, not `'outer` loop labels.
            if (line[i + 2] === "'" || (line[i + 1] === '\\' && line[i + 3] === "'")) {
                inSingle = !inSingle;
            }
        } else if (ch === '/' && line[i + 1] === '/' && !inDouble && !inSingle) {
            return line.slice(0, i);
        }
    }
    return line;
}

/** Collects `domain Name = ...` declarations in the document. */
function declaredDomains(lines: readonly string[]): Set<string> {
    const domains = new Set<string>();
    for (const raw of lines) {
        const line = stripComment(raw);
        for (const m of line.matchAll(/\bdomain\s+([A-Z][A-Za-z0-9_]*)\b/g)) {
            domains.add(m[1]);
        }
    }
    return domains;
}

/** Extracts a balanced generic type name like `SegTree<V, Int>` or `LazySegTree<t.Pre, Int>` at the start of `rhs`. */
function inferConstructorType(rhs: string): string | undefined {
    const trimmed = rhs.trim();
    // Static factory calls like `Graph.undirected(...)`, `Graph.directed(...)`, `Graph.tree(...)`.
    const factory = /^([A-Z][A-Za-z0-9_]*)\.[a-z_][A-Za-z0-9_]*\s*\(/.exec(trimmed);
    if (factory && KNOWN_TYPES.has(factory[1])) {
        return factory[1];
    }
    // Generic constructor calls like `SegTree<V, Int>(...)`, `Marks<D>()`, `Rooted<V>(...)`.
    const head = /^([A-Z][A-Za-z0-9_]*)(\s*<)?/.exec(trimmed);
    if (head && KNOWN_TYPES.has(head[1])) {
        const name = head[1];
        if (head[2]) {
            let depth = 0;
            let i = head[0].length - 1;
            for (; i < trimmed.length; i++) {
                if (trimmed[i] === '<') {
                    depth += 1;
                } else if (trimmed[i] === '>') {
                    depth -= 1;
                    if (depth === 0) {
                        break;
                    }
                }
            }
            if (depth === 0 && /^\s*\(/.test(trimmed.slice(i + 1))) {
                return trimmed.slice(0, i + 1);
            }
            return undefined;
        }
        if (/^\s*\(/.test(trimmed.slice(name.length))) {
            return name;
        }
    }
    // Helper functions with an unmistakable return type, e.g. `compress(all)` -> `Compressed`.
    const fnCall = /^([a-zA-Z_][A-Za-z0-9_]*)\s*\(/.exec(trimmed);
    if (fnCall && fnCall[1] in CONSTRUCTOR_FUNCS) {
        return CONSTRUCTOR_FUNCS[fnCall[1]];
    }
    return undefined;
}

/**
 * Pure inlay-hint computation for a `.cp` document. Emits concise type hints on:
 * - unannotated `let` / `var` bindings initialized from a known constructor or `compress(...)`,
 * - `for x in D:` / `for mut x in D:` loops iterating over a declared `domain D`.
 */
export function computeInlayHints(text: string): HintSpec[] {
    const lines = text.split('\n');
    const domains = declaredDomains(lines);
    const hints: HintSpec[] = [];

    lines.forEach((raw, lineIndex) => {
        const line = stripComment(raw);

        // Unannotated `let x = ...` or `var x = ...` (skip lines with `: Type =`).
        const bind = /^(\s*(?:let|var)\s+)([a-z_][A-Za-z0-9_]*)(\s*=\s*)(.+)$/.exec(line);
        if (bind) {
            const inferred = inferConstructorType(bind[4]);
            if (inferred) {
                hints.push({
                    line: lineIndex,
                    character: bind[1].length + bind[2].length,
                    label: `: ${inferred}`,
                    tooltip: `Inferred type \`${inferred}\``,
                });
            }
            return;
        }

        // `for v in V:` or `for mut v in V:` where `V` is a declared domain.
        const loop = /^(\s*for\s+(?:mut\s+)?)([a-z_][A-Za-z0-9_]*)\s+in\s+([A-Z][A-Za-z0-9_]*)\s*:/.exec(line);
        if (loop && domains.has(loop[3])) {
            hints.push({
                line: lineIndex,
                character: loop[1].length + loop[2].length,
                label: `: ${loop[3]}`,
                tooltip: `Bounded index of \`domain ${loop[3]}\``,
            });
        }
    });

    return hints;
}
