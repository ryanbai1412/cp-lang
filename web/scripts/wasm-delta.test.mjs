import { describe, expect, it } from 'vitest';
import { sizeDelta } from './wasm-delta.mjs';

const was = '789d6e39b0123456789';
const head = 'd76200c7e0123456789';

describe('wasm-size delta', () => {
  it('names the commit range a run grew over', () => {
    expect(sizeDelta(was, 1_886_455, head, 1_892_864)).toEqual({
      delta: 'wasm-size: +6,409 bytes gzipped since 789d6e39b (git log 789d6e39b..d76200c7e)',
      grew: true,
    });
  });
  it('warns only over 1 KB of growth', () => {
    expect(sizeDelta(was, 1_000_000, head, 1_001_000).grew).toBe(false);
    expect(sizeDelta(was, 1_000_000, head, 1_001_001).grew).toBe(true);
  });
  it('reports a cut as a negative change', () => {
    expect(sizeDelta(was, 1_887_462, head, 1_864_048)).toEqual({
      delta: 'wasm-size: -23,414 bytes gzipped since 789d6e39b (git log 789d6e39b..d76200c7e)',
      grew: false,
    });
  });
});
