// The worker behind `spawn()` in cpl.js: loads the compiler once and answers
// one message per call, `{ id, method, args }` with `{ id, result }` or
// `{ id, error: { kind, message } }`.

import { load } from "./cpl.js";

const METHODS = new Set(["check", "lint", "run", "trace", "replay", "stress", "build", "hover", "definition", "symbols", "lsp"]);
const cpl = load();

self.onmessage = async ({ data: { id, method, args } }) => {
  try {
    if (!METHODS.has(method)) throw new Error(`cpl: unknown method \`${method}\``);
    self.postMessage({ id, result: (await cpl)[method](...args) });
  } catch (error) {
    self.postMessage({ id, error: { kind: error.kind ?? "bug", message: error.message } });
  }
};
