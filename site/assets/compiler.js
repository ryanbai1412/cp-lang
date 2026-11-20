// Wrapped with workerSource and packedWasm by site/compiler.py. Shared by all
// offline pages through the parent window; compile/decompress only on first use.
let modulePromise, worker, workerURL, nextId = 0, generation = 0;
const pending = new Map();
function stop(message = "Program stopped.") {
  generation++;
  worker?.terminate();
  worker = null;
  for (const {reject, timer} of pending.values()) {
    clearTimeout(timer);
    reject(new Error(message));
  }
  pending.clear();
}
function moduleBytes() {
  if (!modulePromise) {
    modulePromise = (async () => {
      const bytes = Uint8Array.from(atob(packedWasm), c => c.charCodeAt(0));
      const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
      return WebAssembly.compile(await new Response(stream).arrayBuffer());
    })().catch(error => { modulePromise = null; throw error; });
  }
  return modulePromise;
}
async function call(method, ...args) {
  const current = generation;
  const wasm = await moduleBytes();
  if (generation !== current) throw new Error("Program stopped.");
  return new Promise((resolve, reject) => {
    let initialize = false;
    if (!worker) {
      workerURL ??= URL.createObjectURL(new Blob([workerSource], {type:"text/javascript"}));
      worker = new Worker(workerURL);
      initialize = true;
      worker.onmessage = ({data:{id, result, error}}) => {
        const entry = pending.get(id);
        if (!entry) return;
        pending.delete(id);
        clearTimeout(entry.timer);
        if (error) entry.reject(new Error(error.message));
        else entry.resolve(result);
      };
      worker.onerror = event => {
        event.preventDefault();
        stop("Could not start the compiler: " + event.message);
      };
    }
    const id = nextId++;
    const timer = setTimeout(() => stop("Stopped after 5 seconds. Simplify the program or input and try again."), 5000);
    pending.set(id, {resolve, reject, timer});
    worker.postMessage({id, method, args, ...(initialize ? {wasm} : {})});
  });
}
globalThis.CpCompiler = {call, stop};
addEventListener("pagehide", () => {
  stop();
  if (workerURL) URL.revokeObjectURL(workerURL);
  workerURL = null;
});
