import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import {
    clip,
    parseRunReport,
    parseTestReport,
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

test('labels are clipped to the configured length', () => {
    assert.equal(clip('[1,\n  2]', 20), '[1, 2]');
    assert.equal(clip('abcdefghij', 5), 'abcd\u2026');
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
