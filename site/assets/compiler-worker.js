// Appended to the compiler bridge by site/compiler.py. No fetches or imports.
let compiler;
self.onmessage = async ({data: {id, method, args, wasm}}) => {
  try {
    if (wasm) compiler = load(wasm);
    if (!["check", "run", "trace", "replay", "build", "hover"].includes(method)) throw new Error("Unknown compiler command");
    self.postMessage({id, result:(await compiler)[method](...args)});
  } catch (error) {
    self.postMessage({id, error:{kind:error.kind ?? "bug", message:error.message}});
  }
};
