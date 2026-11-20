/* Explicit example implementations, never eval or a pretend compiler. BigInt
   preserves Int results beyond JavaScript's safe-number range. */
const ExampleRunner = (() => {
  const defaults = {sum:"5\n3 1 4 1 5", bounds:"5\n3 1 4 1 5", accumulation:"4\n0 1 2 3", ascii:"5\nabaca", optional:"", memo:"4"};
  function run(name, input) {
    const tokens = input.trim() ? input.trim().split(/\s+/) : [];
    let position = 0;
    const take = () => {
      if (position >= tokens.length) throw new Error("Not enough input values");
      return tokens[position++];
    };
    const integer = () => {
      const token = take();
      if (!/^[+-]?\d+$/.test(token)) throw new Error("Expected an integer");
      const n = BigInt(token);
      if (n < -(2n ** 63n) || n > 2n ** 63n - 1n) throw new Error("Integer is outside the signed 64-bit range");
      return n;
    };
    const size = max => {
      const n = integer();
      if (n < 1n || n > BigInt(max)) throw new Error("Size must be between 1 and " + max);
      return Number(n);
    };
    let output;
    if (name === "optional") {
      output = "0\n10\n10";
    } else if (name === "memo") {
      const n = size(20);
      let value = 1n;
      for (let i = 1; i < n; i++) value = value * 3n + BigInt(i);
      output = String(value);
    } else if (name === "ascii") {
      const n = size(100), word = take();
      if (word.length !== n || !/^[\x00-\x7f]+$/.test(word)) throw new Error("Expected exactly n ASCII characters");
      output = String([...word].filter(c => c === "a").length);
    } else if (["sum", "bounds", "accumulation"].includes(name)) {
      const n = size(name === "sum" ? 200000 : 100);
      const a = Array.from({length:n}, () => integer());
      if (name === "accumulation" && a.some(v => v < 0n || v > 1000n)) throw new Error("Array values must be between 0 and 1000");
      let total = 0n;
      for (const value of a) {
        total += value;
        if (total < -(2n ** 63n) || total > 2n ** 63n - 1n) throw new Error("Sum overflows signed 64-bit Int");
      }
      output = String(total);
    } else throw new Error("Unknown example");
    if (position !== tokens.length) throw new Error("Unexpected extra input");
    return output + "\n";
  }
  return {run, defaults};
})();
if (typeof document !== "undefined" && document.getElementById("playground-examples")) {
  const examples = JSON.parse(document.getElementById("playground-examples").textContent);
  const picker = document.getElementById("example-picker");
  const input = document.getElementById("example-input");
  const output = document.getElementById("example-output");
  const status = document.getElementById("runner-status");
  function select() {
    const example = examples.find(e => e.id === picker.value);
    document.getElementById("example-source").textContent = example.source;
    document.getElementById("example-file").textContent = example.id + ".cp · read only";
    input.value = ExampleRunner.defaults[example.id];
    input.disabled = example.id === "optional";
    output.textContent = "Run an example to see its output";
    status.textContent = "Ready · Ctrl / ⌘ + Enter to run";
    status.dataset.error = "false";
  }
  function run() {
    try {
      output.textContent = ExampleRunner.run(picker.value, input.value);
      status.textContent = "Completed locally · JavaScript example runner";
      status.dataset.error = "false";
    } catch (error) {
      output.textContent = "";
      status.textContent = error.message;
      status.dataset.error = "true";
    }
  }
  picker.addEventListener("change", select);
  document.getElementById("run-example").addEventListener("click", run);
  document.addEventListener("keydown", event => {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault(); run();
    }
  });
  select();
}
