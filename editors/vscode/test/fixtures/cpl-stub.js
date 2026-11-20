#!/usr/bin/env node
// A stand-in for the parts of `cpl` that are not built yet, for developing
// the editor's overlays. `run` and `stress` on the fixture `d.cp` answer
// with the JSON the extension expects (README.md, "What the extension asks
// of cpl"); every other command goes to the real `cpl` named by `CPL_REAL`.

const { spawnSync } = require('node:child_process');
const fs = require('node:fs');

const args = process.argv.slice(2);
const file = args[1];
const source = file && fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : '';
const canned = (args[0] === 'run' || args[0] === 'stress') && source.includes('fn count_brute()');

if (!canned) {
    const real = process.env.CPL_REAL;
    if (!real) {
        console.error('cpl-stub: set CPL_REAL to the real cpl binary');
        process.exit(2);
    }
    const result = spawnSync(real, args, { stdio: 'inherit' });
    process.exit(result.status ?? 1);
}

/** The 1-based line of the first line matching `pattern`. */
function lineOf(pattern) {
    return source.split('\n').findIndex(line => pattern.test(line)) + 1;
}

const input = fs.readFileSync(0, 'utf8');
if (args[0] === 'stress') {
    console.log(JSON.stringify({
        status: 'fail',
        cases: 313,
        counterexample: {
            case: 313,
            input: 'n = 2  k = 0  a = [7, 7]',
            stages: [{ name: 'count', line: lineOf(/^fn count\(\)/), fast: '1', brute: '0' }],
            saved: 'i3',
        },
    }));
} else if (args.includes('--trace')) {
    console.log('<!doctype html><html><body><h1>Trace of d.cp</h1><p>count() -> 4</p></body></html>');
} else {
    const first = input.startsWith('3 4');
    console.log(JSON.stringify({
        values: [
            { line: lineOf(/^let \(n, k\)/), name: '(n, k)', value: first ? '(3, 4)' : '(2, 0)' },
            { line: lineOf(/^let a:/), name: 'a', value: first ? '[1, 2, 3]' : '[7, 7]' },
            { line: lineOf(/^var pre:/), name: 'pre', value: first ? '[0, 1, 3, 6]' : '[0, 7, 14]' },
            { line: lineOf(/^let answer/), name: 'answer', value: first ? '4' : '0' },
        ],
        stdout: first ? '4\n' : '0\n',
        time_ms: 3,
    }));
}
