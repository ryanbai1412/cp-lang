import { expect, test } from 'vitest';
import { parseExample } from './doc-examples.mjs';

test('an example states the stdin it reads after its code', () => {
  expect(parseExample('`read_all<Int>() == [3, 1, 2]` (stdin `3 1 2`)')).toEqual({ setup: [], expr: 'read_all<Int>()', value: '[3, 1, 2]', stdin: '3 1 2' });
  expect(parseExample('`read_all<Int>() == []` (empty stdin)').stdin).toBe('');
  expect(parseExample('`let x = 1; x + 1 == 2`')).toEqual({ setup: ['let x = 1'], expr: 'x + 1', value: '2', stdin: null });
});
