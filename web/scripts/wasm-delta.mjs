// What scripts/wasm-size.mjs says about growth between main runs (site/R7).
const n = (x) => x.toLocaleString('en-US');

// The change since the last main run's size, and whether it is over 1 KB.
export function sizeDelta(was, wasGz, head, gz) {
  const d = gz - wasGz;
  const range = `${was.slice(0, 9)}..${head.slice(0, 9)}`;
  return { delta: `wasm-size: ${d >= 0 ? '+' : ''}${n(d)} bytes gzipped since ${was.slice(0, 9)} (git log ${range})`, grew: d > 1000 };
}
