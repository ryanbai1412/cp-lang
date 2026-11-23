// The worker behind `spawn()` in cpl.js: loads cpl.wasm on the first call,
// cpl-build.wasm on the first that needs it, and answers one message per call, `{ id, method, args }` with `{ id, result }` or
// `{ id, error: { kind, message } }`.

import { BUILD_CALLS, BUILD_WASM, load } from "./cpl.js";

const METHODS = new Set(["check", "lint", "run", "trace", "replay", "stress", "build", "explain", "hover", "definition", "symbols", "lsp"]);
const BUILD = new Set(BUILD_CALLS);
let cpl = null;
let full = null;
const compiler = (method) => (BUILD.has(method) ? (full ??= load(BUILD_WASM)) : (cpl ??= load()));

self.onmessage = async ({ data: { id, method, args } }) => {
  try {
    if (!METHODS.has(method)) throw new Error(`cpl: unknown method \`${method}\``);
    self.postMessage({ id, result: (await compiler(method))[method](...args) });
  } catch (error) {
    self.postMessage({ id, error: { kind: error.kind ?? "bug", message: error.message } });
  }
};
