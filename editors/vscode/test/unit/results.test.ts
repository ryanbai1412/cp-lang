import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import {
    clip,
    counterexampleSummary,
    divergenceLabel,
    firstDifference,
    parseRunReport,
    parseStressReport,
    parseTestReport,
    renderCounterexample,
    stageLabel,
} from '../../src/results';

test('parseRunReport keeps the view as cpl sends it', () => {
    const view = { panel: '6\n', lines: [{ line: 0, kind: 'live', label: 'i1: 2' }] };
    const report = parseRunReport(JSON.stringify({ values: [], stdout: '6\n', time_ms: 4, view }));
    assert.deepEqual(report, { view, stdout: '6\n', timeMs: 4 });
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
    case_count: 313,
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
    case_count: 17,
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
    assert.deepEqual(parseStressReport('{"status": "pass", "case_count": 1000}'), { status: 'pass', cases: 1000 });
    assert.deepEqual(parseStressReport('{"status": "ok", "case_count": 500}'), { status: 'pass', cases: 500 });
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
        case_count: 3,
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
    const [agree, differ] = parseStressReport(FAIL)!.counterexample!.stages;
    assert.equal(stageLabel(agree, 60), 'fast = brute = [none, 1]');
    assert.equal(stageLabel(differ, 60), 'fast [0]  \u2260  brute [1]  \u2190 first difference');
});

test('parseTestReport reads each case, with null fields left out', () => {
    const cases = parseTestReport(
        JSON.stringify({
            cases: [
                { name: '1', verdict: 'ok', time_ms: 25, expected: '4\n', output: '4\n', why: null, explained: null },
                { name: '2', verdict: 'WA', time_ms: 19, expected: '7\n', output: '6\n', why: 'expected `7`, got `6`', explained: null },
                { name: '3', verdict: null, time_ms: 3, expected: null, output: '9\n', why: null, explained: null },
                { verdict: 'ok' },
            ],
            passed_count: 1,
            case_count: 3,
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
