import { expect, test } from 'vitest';
import { staleReason } from './browser-build.mjs';

test('a browser build is used only when it says it was built from this tree', () => {
  expect(staleReason({ missing: undefined, stamp: 'k1', key: 'k1' })).toBeNull();
  expect(staleReason({ missing: undefined, stamp: 'k0', key: 'k1' })).toMatch(/other sources/);
  expect(staleReason({ missing: undefined, stamp: null, key: 'k1' })).toMatch(/does not say/);
  expect(staleReason({ missing: 'cpl-build.wasm', stamp: 'k1', key: 'k1' })).toMatch(/cpl-build.wasm is missing/);
});
