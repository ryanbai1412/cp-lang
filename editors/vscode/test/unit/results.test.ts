import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import {
    clip,
    parseRunReport,
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
