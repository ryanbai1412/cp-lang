// Generates src/languageTables.ts (the prelude and stdlib name tables) and
// syntaxes/cp.tmLanguage.json (syntaxes/cp.tmLanguage.in.json with those names
// and the literal words of crates/cpl-syntax/src/token.rs filled in) from the
// compiler's prelude (crates/cpl-check/src/prelude.rs) and std/*.cp. Neither is
// committed (decision 572): `npm run compile`, `bundle` and the site's install
// and build run this, so they can't lag the stdlib. It also stamps the workspace
// version from Cargo.toml into package.json; `--check` only reports that.

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
        BUILTIN_VALUES: listed([...builtinValues].filter(name => !builtinTypes.has(name))),
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

const TABLE_DOCS = {
    PRELUDE_TYPES: "/**\n * Types the compiler itself knows (`crates/cpl-check/src/prelude.rs`). `Opt` (spelled `?T`)\n * and `Counter` (what `counts` returns) are left out: neither is written by name.\n */",
    STDLIB_TYPES: "/** Every public type and trait declared in `std/*.cp`, whether `std/prelude.cp` imports it or not. */",
    BUILTIN_VALUES: "/** Functions the compiler itself knows (`crates/cpl-check/src/prelude.rs`), scoped built in. */",
    PRELUDE_VALUES: "/**\n * Functions a program can call without a `use`: the compiler's own (`prelude.rs`) and the\n * ones `std/prelude.cp` imports.\n */",
    STDLIB_VALUES: "/** Public `std/*.cp` functions that need a `use` (`use std.strings.{z_function}`). */",
    PRELUDE_CONSTANTS: "/** Public `std/*.cp` constants the prelude imports (no INF sentinel; use ?Int / None). */",
    STDLIB_CONSTANTS: "/** Public `std/*.cp` constants that need a `use`. */",
};

/** The generated tables module: each table under its doc comment. */
const tablesModule = t => '// Generated by scripts/language-data.js; not committed (decision 572).\n\n'
    + Object.entries(t).map(([name, names]) => `${TABLE_DOCS[name]}\nexport const ${name}: readonly string[] = [\n${tsArray(names)}\n];\n`).join('\n');

/** Longest first, so the regex alternation never stops at a prefix. */
const alternation = names => [...names].sort((a, b) => b.length - a.length || (a < b ? -1 : 1)).join('|');

function main() {
    const check = process.argv.includes('--check');
    const t = tables();
    let grammar = read(path.join(extension, 'syntaxes', 'cp.tmLanguage.in.json'));
    const literals = rustStrings(read(path.join(repo, 'crates', 'cpl-syntax', 'src', 'token.rs')),
        /pub const LITERAL_WORDS: &\[&str\] = &\[/, '];');
    const scopes = {
        'constant.language.boolean.$1.cp': literals.filter(word => word === 'true' || word === 'false'),
        'constant.language.none.cp': literals.filter(word => word !== 'true' && word !== 'false'),
        'support.type.builtin.cp': t.PRELUDE_TYPES,
        'support.type.stdlib.cp': t.STDLIB_TYPES,
        // Only the compiler's own names are built in; a std fn reads as std even when the prelude imports it.
        'support.function.builtin.cp': t.BUILTIN_VALUES,
        'support.function.stdlib.cp': [...t.PRELUDE_VALUES.filter(name => !t.BUILTIN_VALUES.includes(name)), ...t.STDLIB_VALUES],
        'support.constant.stdlib.cp': [...t.PRELUDE_CONSTANTS, ...t.STDLIB_CONSTANTS],
    };
    for (const [scope, names] of Object.entries(scopes)) {
        const rule = new RegExp(`("name": "${scope.replace(/[.$]/g, '\\$&')}",\\s*"match": "[^"]*?\\()GENERATED(\\))`);
        if (!rule.test(grammar)) throw new Error(`no ${scope} rule with (GENERATED) in cp.tmLanguage.in.json`);
        grammar = grammar.replace(rule, (_, open, close) => `${open}${alternation(names)}${close}`);
    }
    // The extension's version is the workspace's, so the .vsix says what it ships with.
    const version = /^\[workspace\.package\][^[]*?^version = "([^"]+)"/m.exec(read(path.join(repo, 'Cargo.toml')))[1];
    const versioned = file => read(file).replace(/^(  "version": )"[^"]*"/m, `$1"${version}"`)
        .replace(/("packages": \{\n    "": \{\n      "name": "[^"]*",\n      "version": )"[^"]*"/, `$1"${version}"`);
    const packageFile = path.join(extension, 'package.json');
    const lockFile = path.join(extension, 'package-lock.json');
    // Generated files are rewritten only when they change, so tsc and caches see them unchanged.
    const generated = [
        [path.join(extension, 'src', 'languageTables.ts'), tablesModule(t)],
        [path.join(extension, 'syntaxes', 'cp.tmLanguage.json'), grammar],
    ];
    for (const [file, text] of check ? [] : generated) {
        if (!fs.existsSync(file) || read(file) !== text) fs.writeFileSync(file, text);
    }
    const stale = [
        [packageFile, versioned(packageFile)],
        [lockFile, versioned(lockFile)],
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
