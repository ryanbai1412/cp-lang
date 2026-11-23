import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vm from 'node:vm';
import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import {
    clip,
    counterexampleSummary,
    divergenceLabel,
    diffOutput,
    firstDifference,
    liveLabel,
    parseCliDiagnostics,
    parseRunReport,
    parseStressReport,
    parseSubmitReport,
    parseTestReport,
    submitSummary,
    parseTraceMessage,
    renderCounterexample,
    runtimeErrorLabel,
    sampleStatusLabel,
    stageLabel,
} from '../../src/results';

test('parseRunReport reads values with 0-based lines', () => {
    const report = parseRunReport(
        JSON.stringify({
            values: [
                { line: 12, name: 'dist', value: '[0, 3, none, 7, 5]' },
                { line: 0, name: 'bad', value: 'x' },
                { name: 'no line', value: 'x' },
            ],
            stdout: '0 3 -1 7 5\n',
            time_ms: 4,
        }),
    );
    assert.deepEqual(report, {
        values: [{ line: 11, name: 'dist', value: '[0, 3, none, 7, 5]' }],
        stdout: '0 3 -1 7 5\n',
        timeMs: 4,
    });
});

test('parseRunReport keeps a runtime error and rejects other output', () => {
    const report = parseRunReport('{"values": [], "error": {"line": 24, "message": "index out of range"}}');
    assert.deepEqual(report?.error, { line: 23, message: 'index out of range' });
    assert.equal(parseRunReport('6\n'), undefined);
    assert.equal(parseRunReport('[1, 2]'), undefined);
    assert.equal(parseRunReport(''), undefined);
});

const FAIL = JSON.stringify({
    status: 'fail',
    cases: 313,
    counterexample: {
        case: 313,
        input: 'n = 2  a = [7, 7]  q = 1  qs = [(l: 2, r: 2)]',
        stages: [
            { name: 'last_seen', line: 9, fast: '[none, 1]', brute: '[none, 1]' },
            { name: 'answer', line: 14, fast: '[0]', brute: '[1]' },
        ],
        saved: 'i6',
    },
});

const CLI_FAIL = JSON.stringify({
    status: 'fail',
    cases: 17,
    counterexample: {
        case: 17,
        seed: 42,
        verdict: 'wrong_answer',
        input: 'n = 2  a = [7, 7]',
        raw_input: '2\n7 7\n',
        expected: '1\n',
        got: '0\n',
        stages: [],
        saved: 'd.tests/stress-17.in',
    },
});

test('parseStressReport reads stage diffs, ok status, and whole-program CLI reports', () => {
    const report = parseStressReport(FAIL);
    assert.equal(report?.status, 'fail');
    const found = report!.counterexample!;
    assert.deepEqual(found.stages.map(s => [s.name, s.line, s.differs]), [
        ['last_seen', 8, false],
        ['answer', 13, true],
    ]);
    assert.equal(firstDifference(found)?.name, 'answer');
    assert.deepEqual(parseStressReport('{"status": "pass", "cases": 1000}'), { status: 'pass', cases: 1000 });
    assert.deepEqual(parseStressReport('{"status": "ok", "cases": 500}'), { status: 'pass', cases: 500 });
    assert.equal(parseStressReport('312 cases ... FAIL'), undefined);

    const cli = parseStressReport(CLI_FAIL);
    assert.equal(cli?.status, 'fail');
    assert.equal(cli?.counterexample?.seed, 42);
    assert.equal(cli?.counterexample?.verdict, 'wrong_answer');
    assert.equal(cli?.counterexample?.expected, '1\n');
    assert.equal(cli?.counterexample?.got, '0\n');
    assert.equal(
        counterexampleSummary(cli!.counterexample!, 60),
        'counterexample: n = 2 a = [7, 7]  (fast 0 \u2260 brute 1)',
    );
});

test('renderCounterexample lays out stage diffs and whole-program diffs', () => {
    assert.equal(
        renderCounterexample(parseStressReport(FAIL)!.counterexample!),
        [
            'FAIL at case 313, shrunk to:',
            '  n = 2  a = [7, 7]  q = 1  qs = [(l: 2, r: 2)]',
            '  part       fast       brute',
            '  last_seen  [none, 1]  [none, 1]',
            '  answer     [0]        [1]   <- first difference',
            '  saved as i6',
        ].join('\n'),
    );
    assert.equal(
        renderCounterexample(parseStressReport(CLI_FAIL)!.counterexample!),
        [
            'FAIL at case 17 (seed 42) [wrong_answer], shrunk to:',
            '  n = 2  a = [7, 7]',
            '  fast:  0',
            '  brute: 1',
            '  saved as d.tests/stress-17.in',
        ].join('\n'),
    );
});

test('a stress report with a wrong part call shows that call', () => {
    const json = JSON.stringify({
        status: 'fail',
        cases: 3,
        counterexample: {
            case: 3, seed: 1, input: 'k = 1', raw_input: '1\n', expected: '1\n', got: '0\n', verdict: 'WA', stages: [], saved: null, trace: null,
            explain: {
                divergence: {
                    name: 'tri(k: 1)', nth: 1, call: true,
                    fast: { line: 4, value: '0' }, brute: { line: 10, value: '1' },
                    inside: [{ line: 5, locals: 'k: 1, s: 0' }],
                },
            },
        },
    });
    const found = parseStressReport(json)!.counterexample!;
    assert.deepEqual(found.divergence?.fast, { line: 3, value: '0' });
    assert.equal(divergenceLabel(found.divergence!, 40), 'tri(k: 1) = 0  \u2260  brute 1  \u2190 first wrong call');
    assert.equal(
        renderCounterexample(found),
        [
            'FAIL at case 3 (seed 1) [WA], shrunk to:',
            '  k = 1',
            '  fast:  0',
            '  brute: 1',
            '  first wrong call: `tri(k: 1)`, the 1st call of `tri`',
            '    solution tri(k: 1) = 0 at line 4',
            '    brute    tri(k: 1) = 1 at line 10',
            "    the solution's last steps in it:",
            '      line 5: k: 1, s: 0',
        ].join('\n'),
    );
    const none = JSON.parse(json);
    none.counterexample.explain = { divergence: null };
    assert.equal(parseStressReport(JSON.stringify(none))!.counterexample!.divergence, undefined);
});

test('labels are clipped to the configured length', () => {
    assert.equal(clip('[1,\n  2]', 20), '[1, 2]');
    assert.equal(clip('abcdefghij', 5), 'abcd\u2026');
    assert.equal(liveLabel('i1', [{ line: 3, name: 'dist', value: '[0, 3, none]' }], 60), 'i1: [0, 3, none]');
    assert.equal(
        liveLabel('i1', [{ line: 3, name: 'n', value: '2' }, { line: 3, name: 'm', value: '5' }], 60),
        'i1: n = 2  m = 5',
    );
    const [agree, differ] = parseStressReport(FAIL)!.counterexample!.stages;
    assert.equal(stageLabel(agree, 60), 'fast = brute = [none, 1]');
    assert.equal(stageLabel(differ, 60), 'fast [0]  \u2260  brute [1]  \u2190 first difference');
});

test('diffOutput ignores trailing spaces and blank lines, and finds the first different line', () => {
    assert.deepEqual(diffOutput('1 2 \n3\n\n', '1 2\n3'), { same: true });
    assert.deepEqual(diffOutput('1\n2\n3\n', '1\n5\n3\n'), { same: false, line: 1, expected: '2', actual: '5' });
    assert.deepEqual(diffOutput('1\n', '1\n2\n'), { same: false, line: 1, expected: undefined, actual: '2' });
});

test('parseCliDiagnostics extracts line, column, code and help from cpl stderr', () => {
    const stderr = [
        'cpl: the solution failed',
        'error[E0301]: `deg` is indexed by V, but `e` is an E',
        '  --> d.cp:7:17',
        '  |',
        '7 | let x = deg[e]',
        '  |             ^',
        '  = help: index with a V',
        '',
        'warning[W0002]: unused binding `tmp`',
        '  --> d.cp:3:5',
    ].join('\n');
    assert.deepEqual(parseCliDiagnostics(stderr), [
        {
            severity: 'error',
            code: 'E0301',
            message: '`deg` is indexed by V, but `e` is an E',
            file: 'd.cp',
            line: 6,
            col: 16,
            help: 'index with a V',
        },
        {
            severity: 'warning',
            code: 'W0002',
            message: 'unused binding `tmp`',
            file: 'd.cp',
            line: 2,
            col: 4,
        },
    ]);
});

test('runtimeErrorLabel and sampleStatusLabel format compact inline badges', () => {
    assert.equal(
        runtimeErrorLabel({ line: 10, code: 'E9001', message: 'index out of range', sample: 'i1' }, 60),
        'i1 error: [E9001] index out of range',
    );
    assert.equal(
        runtimeErrorLabel({ line: 4, message: 'assertion failed' }, 60),
        'runtime error: assertion failed',
    );
    assert.equal(sampleStatusLabel({ line: 0, ok: true, label: 'i1: passed (3 ms)' }, 60), '\u2713 i1: passed (3 ms)');
    assert.equal(sampleStatusLabel({ line: 0, ok: false, label: '1 of 2 samples passed' }, 60), '\u2717 1 of 2 samples passed');
});

test('the editor reads what the real trace viewer posts, and nothing else', () => {
    const viewer = fs.readFileSync(path.resolve(__dirname, '../../../../../tools/trace/index.html'), 'utf8');
    const core = viewer.match(/<script id="trace-core">([\s\S]*?)<\/script>/)![1];
    const T = vm.runInNewContext(`${core}\nTraceCore`);
    const posted: unknown[] = [];
    const link = T.editorLink((message: unknown) => posted.push(message));
    link.step(4);
    link.reveal(9);
    assert.deepEqual(
        posted.map(parseTraceMessage),
        [
            { type: 'step', line: 4 },
            { type: 'reveal', line: 9 },
        ],
    );
    for (const bad of [null, 'step', { type: 'step' }, { type: 'step', line: 0 }, { type: 'jump', line: 2 }, { type: 'reveal', line: 1.5 }]) {
        assert.equal(parseTraceMessage(bad), undefined, JSON.stringify(bad));
    }
});

test('parseSubmitReport reads `cpl submit --json` and submitSummary says if it is ready', () => {
    const stdout =
        '{"build":{"bytes":4593,"checks":"none","judge":"codeforces","limit":65536,"path":"a.cpp","sizes":[]},' +
        '"ready":true,"test":{"cases":[],"passed":2,"total":2}}\n';
    const report = parseSubmitReport(stdout);
    assert.deepEqual(report, { ready: true, path: 'a.cpp', bytes: 4593, limit: 65536, judge: 'codeforces', passed: 2, total: 2 });
    assert.equal(submitSummary(report!), "ready to send: a.cpp (4.5 KB, 7% of codeforces's limit), 2/2 samples");
    const failed = parseSubmitReport('{"ready":false,"build":{"path":"a.cpp","bytes":10},"test":{"passed":0,"total":1}}');
    assert.equal(submitSummary(failed!), 'not ready to send: a.cpp (0.0 KB), 0/1 samples');
    assert.equal(parseSubmitReport('wrote a.cpp'), undefined);
    assert.equal(parseSubmitReport('{"cases":[]}'), undefined);
});

test('parseTestReport reads each case, with null fields left out', () => {
    const cases = parseTestReport(
        JSON.stringify({
            cases: [
                { name: '1', verdict: 'ok', ms: 25, expected: '4\n', output: '4\n', why: null, explained: null },
                { name: '2', verdict: 'WA', ms: 19, expected: '7\n', output: '6\n', why: 'expected `7`, got `6`', explained: null },
                { name: '3', verdict: null, ms: 3, expected: null, output: '9\n', why: null, explained: null },
                { verdict: 'ok' },
            ],
            passed: 1,
            total: 3,
        }),
    );
    assert.deepEqual(cases, [
        { name: '1', verdict: 'ok', ms: 25, expected: '4\n', output: '4\n', why: undefined, explained: undefined },
        { name: '2', verdict: 'WA', ms: 19, expected: '7\n', output: '6\n', why: 'expected `7`, got `6`', explained: undefined },
        { name: '3', verdict: undefined, ms: 3, expected: undefined, output: '9\n', why: undefined, explained: undefined },
    ]);
    assert.equal(parseTestReport('{"values": []}'), undefined);
    assert.equal(parseTestReport('error[E0201]'), undefined);
});
