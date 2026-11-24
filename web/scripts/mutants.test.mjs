import { describe, expect, it } from 'vitest';
import { mutants } from './mutants.mjs';

const tos = (src) => mutants(src).map((m) => m.to);

describe('practice mutants', () => {
  it('flips one comparison at a time', () => {
    expect(tos('if a <= b:')).toEqual(['if a < b:']);
    expect(tos('if a == b and c:')).toEqual(['if a != b and c:', 'if a == b or c:']);
  });
  it('shifts ranges and constants by one', () => {
    expect(tos('for i in 0..n:')).toEqual(['for i in 0..=n:']);
    expect(tos('let k = 2')).toEqual(['let k = 3']);
    expect(tos('let x = n + 1')).toEqual(['let x = n', 'let x = n - 1', 'let x = n + 2']);
  });
  it('swaps min and max', () => {
    expect(tos('best = min(best, x)')).toEqual(['best = max(best, x)']);
  });
  it('leaves input, signatures, strings and comments alone', () => {
    expect(tos('let n: Int = read()')).toEqual([]);
    expect(tos('fn f(a: Int) -> Int')).toEqual([]);
    expect(tos('print("a <= b")')).toEqual([]);
    expect(tos('x = 1 // was 2')).toEqual(['x = 2 // was 2']);
  });
  it('rebuilds the whole program around the changed line', () => {
    const [m] = mutants('let a = 3\nprint(a)');
    expect(m).toMatchObject({ line: 1, from: 'let a = 3', to: 'let a = 4', src: 'let a = 4\nprint(a)' });
  });
});
