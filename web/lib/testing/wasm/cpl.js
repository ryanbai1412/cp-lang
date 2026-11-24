// Stands in for the site's /wasm/cpl.js in unit tests: cpl.ts imports
// `${location.origin}/wasm/cpl.js`, and a test points location here and sets
// globalThis.spawnFakeCpl to make the worker it gets.
export const spawn = (options) => globalThis.spawnFakeCpl(options);
