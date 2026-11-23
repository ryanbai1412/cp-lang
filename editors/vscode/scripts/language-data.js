// Regenerates the prelude and stdlib name tables in src/languageData.ts and
// the matching alternations in syntaxes/cp.tmLanguage.json from the compiler's
// prelude (crates/cpl-check/src/prelude.rs) and std/*.cp, so the extension
// never lags the stdlib. `npm run compile` runs it; `--check` only reports.

const fs = require('node:fs');
const path = require('node:path');

const extension = path.resolve(__dirname, '..');
const repo = path.resolve(extension, '..', '..');
const read = file => fs.readFileSync(file, 'utf8').replace(/\r\n/g, '\n');

/** Names the compiler's prelude knows that the tables leave out on purpose (`Opt` is spelled `?T`). */
const UNLISTED = new Set(['INF', 'Opt', 'Counter', 'argmax', 'argmin', 'columns', 'last', 'take']);

function rustStrings(source, from, end) {
    const start = source.search(from);
    if (start < 0) throw new Error(`no ${from} in prelude.rs`);
    const body = source.slice(start, source.indexOf(end, start));
    return [...body.matchAll(/"([A-Za-z_][A-Za-z0-9_]*)"/g)].map(m => m[1]);
}

function tables() {
    const prelude = read(path.join(repo, 'crates', 'cpl-check', 'src', 'prelude.rs'));
    const typeNames = rustStrings(prelude, /const TYPE_NAMES: &\[&str\] = &\[/, '];');
    const builtinTypes = new Set([...rustStrings(prelude, /pub fn ty\(/, '\n}\n'), ...typeNames]);
    const builtinValues = new Set([
        ...rustStrings(prelude, /pub fn value\(/, '\n}\n'),
        ...rustStrings(prelude, /pub const VALUE_NAMES: &\[&str\] = &\[/, '];'),
    ]);
    const std = path.join(repo, 'std');
    const imported = new Set();
    for (const use of read(path.join(std, 'prelude.cp')).matchAll(/^use std\.\w+\.\{([^}]*)\}/gm)) {
        use[1].split(',').map(name => name.trim()).filter(Boolean).forEach(name => imported.add(name));
    }
    const declared = new Map();
    for (const file of fs.readdirSync(std).filter(name => name.endsWith('.cp'))) {
        for (const line of read(path.join(std, file)).split('\n')) {
            const decl = /^(?:@get )?pub (?:@get )?(?:@\w+ )*(fn|struct|enum|trait|type|const|domain|context) ([A-Za-z_][A-Za-z0-9_]*)/.exec(line);
            // A `@get` fn reads as a field, so it is a name of its own only when the prelude imports it;
            // `_` names are the stdlib's helpers and `_of_pairs`/`_of_triples` what `T.fn(list)` resolves to.
            const getter = /^(?:@get )?pub @get |^@get pub /.test(line);
            if (decl && !decl[2].startsWith('_') && (!getter || imported.has(decl[2])) && !/_of_(pairs|triples)$/.test(decl[2])) {
                declared.set(decl[2], decl[1]);
            }
        }
    }
    const kinds = (...wanted) => [...declared].filter(([, kind]) => wanted.includes(kind)).map(([name]) => name);
    const listed = names => [...new Set(names)].filter(name => !UNLISTED.has(name)).sort();
    const names = list => [...new Set(list)].sort();
    const fns = kinds('fn');
    const consts = kinds('const', 'context');
    return {
        PRELUDE_TYPES: listed(builtinTypes),
        STDLIB_TYPES: names(kinds('struct', 'enum', 'trait', 'type', 'domain')),
        PRELUDE_VALUES: listed([...builtinValues, ...fns.filter(name => imported.has(name))].filter(name => !builtinTypes.has(name))),
        STDLIB_VALUES: names(fns.filter(name => !imported.has(name))),
        PRELUDE_CONSTANTS: names(consts.filter(name => imported.has(name))),
        STDLIB_CONSTANTS: names(consts.filter(name => !imported.has(name))),
    };
}

/** `names` as the TypeScript array body languageData.ts uses: quoted, about 100 columns a line. */
function tsArray(names) {
    const lines = [];
    let line = '   ';
    for (const name of names) {
        const word = ` '${name}',`;
        if (line.length + word.length > 100) {
            lines.push(line);
            line = '   ';
        }
        line += word;
    }
    lines.push(line);
    return lines.join('\n');
}

/** Longest first, so the regex alternation never stops at a prefix. */
const alternation = names => [...names].sort((a, b) => b.length - a.length || (a < b ? -1 : 1)).join('|');

function main() {
    const check = process.argv.includes('--check');
    const t = tables();
    const dataFile = path.join(extension, 'src', 'languageData.ts');
    let data = read(dataFile);
    for (const [name, names] of Object.entries(t)) {
        const rule = new RegExp(`(export const ${name}: readonly string\\[\\] = \\[\\n)[\\s\\S]*?(\\n\\];)`);
        if (!rule.test(data)) throw new Error(`no ${name} table in languageData.ts`);
        data = data.replace(rule, (_, open, close) => `${open}${tsArray(names)}${close}`);
    }
    const grammarFile = path.join(extension, 'syntaxes', 'cp.tmLanguage.json');
    let grammar = read(grammarFile);
    const scopes = {
        'support.type.builtin.cp': t.PRELUDE_TYPES,
        'support.type.stdlib.cp': t.STDLIB_TYPES,
        'support.function.builtin.cp': t.PRELUDE_VALUES,
        'support.function.stdlib.cp': t.STDLIB_VALUES,
        'support.constant.stdlib.cp': [...t.PRELUDE_CONSTANTS, ...t.STDLIB_CONSTANTS],
    };
    for (const [scope, names] of Object.entries(scopes)) {
        const rule = new RegExp(`("name": "${scope.replace(/\./g, '\\.')}",\\s*"match": "[^"]*?\\()[A-Za-z0-9_|]+(\\))`);
        if (!rule.test(grammar)) throw new Error(`no ${scope} rule in the grammar`);
        grammar = grammar.replace(rule, (_, open, close) => `${open}${alternation(names)}${close}`);
    }
    const stale = [
        [dataFile, data],
        [grammarFile, grammar],
    ].filter(([file, text]) => read(file) !== text);
    for (const [file, text] of stale) {
        if (check) {
            console.error(`language-data: ${path.relative(repo, file)} is stale; run \`npm run language-data\``);
        } else {
            fs.writeFileSync(file, text);
            console.log(`language-data: updated ${path.relative(repo, file)}`);
        }
    }
    process.exitCode = check && stale.length > 0 ? 1 : 0;
}

if (require.main === module) {
    main();
}
module.exports = { tables };
