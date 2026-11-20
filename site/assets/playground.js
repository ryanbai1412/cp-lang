/* Shared real-compiler editor: all CP source stays text, never HTML or JS. */
(() => {
  "use strict";
  const examples = JSON.parse(document.getElementById("playground-examples")?.textContent ?? "[]");
  document.querySelectorAll("[data-cpl-editor]").forEach(editor => {
    const find = selector => editor.querySelector(selector);
    const source = find("[data-source]"), stdin = find("[data-stdin]");
    const stdout = find("[data-stdout]"), status = find("[data-status]");
    const diagnostics = find("[data-diagnostics]"), hover = find("[data-hover]");
    const run = find("[data-run]"), build = find("[data-build]"), stop = find("[data-stop]");
    let revision = 0, checkTimer, hoverTimer;
    function showDiagnostics(result) {
      diagnostics.replaceChildren();
      for (const diagnostic of result.diagnostics ?? []) {
        const button = document.createElement("button");
        button.type = "button";
        const {line, character} = diagnostic.range.start;
        button.textContent = `${line + 1}:${character + 1} ${diagnostic.code ?? ""} ${diagnostic.message}`;
        button.addEventListener("click", () => {
          source.focus(); source.setSelectionRange(diagnostic.start, diagnostic.end); inspect();
        });
        diagnostics.append(button);
        if (diagnostic.help) {
          const help = document.createElement("p"); help.textContent = diagnostic.help; diagnostics.append(help);
        }
      }
    }
    async function check() {
      const current = revision;
      try {
        const result = await CpCompiler.call("check", source.value);
        if (revision === current && !run.disabled) showDiagnostics(result);
      } catch (error) { if (revision === current && !run.disabled) status.textContent = error.message; }
    }
    function inspect() {
      clearTimeout(hoverTimer);
      const current = revision, offset = source.selectionStart;
      hoverTimer = setTimeout(async () => {
        try {
          const result = await CpCompiler.call("hover", source.value, offset);
          if (revision !== current || offset !== source.selectionStart) return;
          hover.textContent = result?.text ?? "Place the cursor on a name to inspect its type.";
        } catch (error) { if (revision === current) hover.textContent = error.message; }
      }, 180);
    }
    function changed() {
      revision++;
      clearTimeout(checkTimer);
      diagnostics.replaceChildren();
      find("[data-cpp]").textContent = "";
      find("[data-cpp-panel]").open = false;
      status.textContent = "Source changed · Ctrl / ⌘ + Enter to run";
      checkTimer = setTimeout(check, 300);
      inspect();
    }
    async function execute(method) {
      const current = revision;
      clearTimeout(checkTimer); clearTimeout(hoverTimer);
      run.disabled = build.disabled = true; stop.disabled = false;
      status.dataset.error = "false";
      status.textContent = method === "run" ? "Running…" : "Generating C++17…";
      if (method === "run") stdout.textContent = "";
      try {
        const result = await CpCompiler.call(method, source.value, ...(method === "run" ? [stdin.value] : []));
        if (revision !== current) return;
        showDiagnostics(result);
        status.dataset.error = String(!result.ok);
        status.textContent = result.ok ? (method === "run" ? "Completed locally · CP Lang WebAssembly" : "C++17 generated locally")
          : result.stderr || "Compilation failed.";
        if (method === "run") stdout.textContent = result.stdout;
        else {
          find("[data-cpp]").textContent = result.cpp ?? "No C++ generated. Fix the diagnostics above.";
          find("[data-cpp-panel]").open = true;
        }
      } catch (error) {
        if (revision === current) { status.textContent = error.message; status.dataset.error = "true"; }
      } finally {
        run.disabled = build.disabled = false; stop.disabled = true;
      }
    }
    run.addEventListener("click", () => execute("run"));
    build.addEventListener("click", () => execute("build"));
    stop.addEventListener("click", () => CpCompiler.stop());
    source.addEventListener("input", changed);
    source.addEventListener("click", inspect);
    source.addEventListener("keyup", inspect);
    editor.addEventListener("keydown", event => {
      if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && !run.disabled) {
        event.preventDefault(); execute("run");
      }
    });
    const picker = document.getElementById("example-picker");
    if (picker && editor.closest("#playground-editor")) {
      const select = () => {
        const example = examples.find(item => item.id === picker.value);
        source.value = example.source; stdin.value = example.stdin;
        stdout.textContent = "Run a program to see its output.";
        changed();
      };
      picker.addEventListener("change", select);
      select();
    }
  });
})();
