// The worker behind `spawn()` in cpl.js: loads cpl.wasm on the first call,
// cpl-build.wasm on the first that needs it, and answers one message per call, `{ id, method, args }` with `{ id, result }` or
// `{ id, error: { kind, message } }`.

import { BUILD_CALLS, BUILD_WASM, load } from "./cpl.js";

const METHODS = new Set(["stdNeeded", "provideStd", "check", "lint", "run", "trace", "replay", "stress", "sample", "build", "explain", "hover", "definition", "symbols", "lsp"]);
const BUILD = new Set(BUILD_CALLS);
let cpl = null;
let full = null;
// The std modules given with `provideStd`, which both modules get, the
// second one when it loads.
const given = new Map();
const giveAll = (module) => {
  for (const [name, text] of given) module.provideStd(name, text);
  return module;
};
// A failed load is forgotten, so the next call downloads again.
const compiler = (method) =>
  BUILD.has(method)
    ? (full ??= load(BUILD_WASM).then(giveAll).catch((e) => ((full = null), Promise.reject(e))))
    : (cpl ??= load().catch((e) => ((cpl = null), Promise.reject(e))));

const call = async (method, args) => {
  const result = (await compiler(method))[method](...args);
  if (method === "provideStd" && !result.error) {
    given.set(args[0], args[1]);
    if (full) (await full.catch(() => null))?.provideStd(args[0], args[1]);
  }
  return result;
};

self.onmessage = async ({ data: { id, method, args } }) => {
  try {
    if (!METHODS.has(method)) throw new Error(`cpl: unknown method \`${method}\``);
    self.postMessage({ id, result: await call(method, args) });
  } catch (error) {
    self.postMessage({ id, error: { kind: error.kind ?? "bug", message: error.message } });
  }
};
