import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import { clip } from '../../src/results';

test('labels are clipped to the configured length', () => {
    assert.equal(clip('[1,\n  2]', 20), '[1, 2]');
    assert.equal(clip('abcdefghij', 5), 'abcd\u2026');
});
