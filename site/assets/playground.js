/* Shared real-compiler editor: all CP source stays text, never HTML or JS. */
(() => {
  "use strict";

  function encodeShare(payload) {
    const bytes = new TextEncoder().encode(JSON.stringify(payload));
    let bin = "";
    for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  function decodeShare(hash) {
    if (!hash || !hash.startsWith("#share=")) return null;
    try {
      let b64 = hash.slice("#share=".length).replace(/-/g, "+").replace(/_/g, "/");
      while (b64.length % 4) b64 += "=";
      const bin = atob(b64);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      const data = JSON.parse(new TextDecoder().decode(bytes));
      if (!data || typeof data.source !== "string") return null;
      return {
        source: data.source,
        stdin: typeof data.stdin === "string" ? data.stdin : "",
        samples: Array.isArray(data.samples)
          ? data.samples.filter(s => s && typeof s.input === "string" && typeof s.expected === "string")
          : [],
      };
    } catch (_) {
      return null;
    }
  }

  function normalizeOutput(text) {
    return String(text ?? "")
      .replace(/\r\n/g, "\n")
      .trim()
      .split("\n")
      .map(line => line.replace(/[ \t]+$/, ""))
      .join("\n");
  }

  window.CpPlaygroundShare = { encode: encodeShare, decode: decodeShare, normalizeOutput };

  const examples = JSON.parse(document.getElementById("playground-examples")?.textContent ?? "[]");
  document.querySelectorAll("[data-cpl-editor]").forEach(editor => {
    const find = selector => editor.querySelector(selector);
    const source = find("[data-source]"), stdin = find("[data-stdin]");
    const stdout = find("[data-stdout]"), status = find("[data-status]");
    const diagnostics = find("[data-diagnostics]"), cliStderr = find("[data-cli-stderr]");
    const hover = find("[data-hover]");
    const run = find("[data-run]"), build = find("[data-build]"), stop = find("[data-stop]");
    const traceButton = find("[data-trace]");
    const shareButton = find("[data-share]");
    const runSamplesButton = find("[data-run-samples]");
    const runSamplesInner = find("[data-run-samples-inner]");
    const addSampleButton = find("[data-add-sample]");
    const samplesPanel = find("[data-samples-panel]");
    const samplesList = find("[data-samples-list]");
    const samplesBadge = find("[data-samples-badge]");
    let revision = 0, checkTimer, hoverTimer;
    let samples = [];

    function renderSamples() {
      if (!samplesList) return;
      samplesList.replaceChildren();
      if (samplesBadge) samplesBadge.textContent = samples.length ? `(${samples.length})` : "(0)";
      samples.forEach((sample, idx) => {
        const row = document.createElement("div");
        row.className = "sample-row";
        row.dataset.sampleIndex = String(idx);

        const head = document.createElement("div");
        head.className = "sample-row-head";
        const title = document.createElement("span");
        title.className = "sample-title";
        title.textContent = `Sample ${idx + 1}`;
        const verdict = document.createElement("span");
        verdict.className = "sample-verdict";
        verdict.dataset.verdict = "";
        const remove = document.createElement("button");
        remove.type = "button";
        remove.className = "sample-remove";
        remove.textContent = "Remove";
        remove.addEventListener("click", () => {
          samples.splice(idx, 1);
          renderSamples();
        });
        head.append(title, verdict, remove);

        const fields = document.createElement("div");
        fields.className = "sample-fields";
        const inLabel = document.createElement("label");
        inLabel.textContent = "Input";
        const inBox = document.createElement("textarea");
        inBox.spellcheck = false;
        inBox.value = sample.input;
        inBox.addEventListener("input", () => { sample.input = inBox.value; });
        inLabel.append(inBox);

        const expLabel = document.createElement("label");
        expLabel.textContent = "Expected output";
        const expBox = document.createElement("textarea");
        expBox.spellcheck = false;
        expBox.value = sample.expected;
        expBox.addEventListener("input", () => { sample.expected = expBox.value; });
        expLabel.append(expBox);

        fields.append(inLabel, expLabel);
        const detail = document.createElement("pre");
        detail.className = "sample-detail";
        detail.hidden = true;
        row.append(head, fields, detail);
        samplesList.append(row);
      });
    }

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
      if (cliStderr) {
        const text = result.stderr ?? "";
        cliStderr.textContent = text;
        cliStderr.hidden = !text;
      }
    }

    function setBusy(busy) {
      run.disabled = build.disabled = traceButton.disabled = busy;
      if (runSamplesButton) runSamplesButton.disabled = busy;
      if (runSamplesInner) runSamplesInner.disabled = busy;
      stop.disabled = !busy;
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
      if (cliStderr) { cliStderr.textContent = ""; cliStderr.hidden = true; }
      find("[data-cpp]").textContent = "";
      find("[data-cpp-panel]").open = false;
      status.dataset.error = "false";
      status.textContent = "Source changed · Ctrl / ⌘ + Enter to run";
      checkTimer = setTimeout(check, 300);
      inspect();
    }

    async function execute(method) {
      const current = revision;
      clearTimeout(checkTimer); clearTimeout(hoverTimer);
      const runs = method !== "build";
      setBusy(true);
      status.dataset.error = "false";
      status.textContent = method === "run" ? "Running…" : method === "trace" ? "Recording every step…" : "Generating C++17…";
      if (runs) stdout.textContent = "";
      try {
        const result = await CpCompiler.call(method, source.value, ...(runs ? [stdin.value] : []));
        if (revision !== current) return;
        showDiagnostics(result);
        status.dataset.error = String(!result.ok);
        status.textContent = result.ok ? (runs ? "Completed locally · CP Lang WebAssembly" : "C++17 generated locally")
          : result.stderr || "Compilation failed.";
        if (runs) stdout.textContent = result.stdout;
        if (method === "trace" && result.trace) openTrace(result.trace);
        if (!runs) {
          const cppEl = find("[data-cpp]");
          if (result.cpp && typeof window.cpHighlightCpp === "function") {
            window.cpHighlightCpp(cppEl, result.cpp);
          } else {
            cppEl.textContent = result.cpp ?? "No C++ generated. Fix the diagnostics above.";
          }
          find("[data-cpp-panel]").open = true;
        }
      } catch (error) {
        if (revision === current) { status.textContent = error.message; status.dataset.error = "true"; }
      } finally {
        setBusy(false);
      }
    }

    async function runSamples() {
      const current = revision;
      clearTimeout(checkTimer); clearTimeout(hoverTimer);
      if (samplesPanel) samplesPanel.open = true;
      if (!samples.length) {
        samples.push({ input: stdin.value, expected: "" });
        renderSamples();
      }
      setBusy(true);
      status.dataset.error = "false";
      status.textContent = `Running ${samples.length} sample${samples.length === 1 ? "" : "s"}…`;
      let passed = 0;
      let firstError = null;
      try {
        const rows = samplesList ? Array.from(samplesList.querySelectorAll(".sample-row")) : [];
        for (let i = 0; i < samples.length; i++) {
          const sample = samples[i];
          const row = rows[i];
          const verdictEl = row?.querySelector("[data-verdict]");
          const detailEl = row?.querySelector(".sample-detail");
          const result = await CpCompiler.call("run", source.value, sample.input);
          if (revision !== current) return;
          if (!result.ok) {
            if (!firstError) firstError = result;
            if (verdictEl) {
              verdictEl.textContent = result.exitCode === 3 ? "RE" : "CE";
              verdictEl.dataset.state = "fail";
            }
            if (detailEl) {
              detailEl.textContent = result.stderr || "Execution failed.";
              detailEl.hidden = false;
            }
            continue;
          }
          const got = normalizeOutput(result.stdout);
          const want = normalizeOutput(sample.expected);
          if (got === want) {
            passed++;
            if (verdictEl) { verdictEl.textContent = "PASS"; verdictEl.dataset.state = "pass"; }
            if (detailEl) { detailEl.textContent = ""; detailEl.hidden = true; }
          } else {
            if (verdictEl) { verdictEl.textContent = "WA"; verdictEl.dataset.state = "fail"; }
            if (detailEl) {
              detailEl.textContent = `Expected:\n${want}\n\nGot:\n${got}`;
              detailEl.hidden = false;
            }
          }
        }
        if (firstError) showDiagnostics(firstError);
        else showDiagnostics({ diagnostics: [], stderr: "" });
        const allPass = passed === samples.length;
        status.dataset.error = String(!allPass);
        status.textContent = `${passed}/${samples.length} samples passed`;
        if (samplesBadge) samplesBadge.textContent = `(${passed}/${samples.length} passed)`;
      } catch (error) {
        if (revision === current) { status.textContent = error.message; status.dataset.error = "true"; }
      } finally {
        setBusy(false);
      }
    }

    async function shareLink() {
      const encoded = encodeShare({ source: source.value, stdin: stdin.value, samples });
      const hash = `#share=${encoded}`;
      try { history.replaceState(null, "", hash); } catch (_) { location.hash = hash; }
      const url = location.href;
      let copied = false;
      try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          await navigator.clipboard.writeText(url);
          copied = true;
        }
      } catch (_) {}
      status.dataset.error = "false";
      status.textContent = copied
        ? `Shareable link copied (${url.length} chars)`
        : `Shareable link updated in URL bar (${url.length} chars)`;
    }

    function openTrace(trace) {
      const text = JSON.stringify(trace), offline = window.parent !== window;
      try {
        if (offline) { window.parent.cpLiveTrace = text; window.parent.cpLiveInput = stdin.value; }
        else { sessionStorage.setItem("cp-live-trace", text); sessionStorage.setItem("cp-live-input", stdin.value); }
      } catch (_) {
        status.dataset.error = "true";
        status.textContent = "The trace is too large to hand to the debugger. Use a smaller input.";
        return;
      }
      if (offline) window.parent.location.hash = "trace.html#live";
      else location.href = "trace.html#live";
    }

    run.addEventListener("click", () => execute("run"));
    traceButton.addEventListener("click", () => execute("trace"));
    build.addEventListener("click", () => execute("build"));
    stop.addEventListener("click", () => CpCompiler.stop());
    if (shareButton) shareButton.addEventListener("click", shareLink);
    if (runSamplesButton) runSamplesButton.addEventListener("click", runSamples);
    if (runSamplesInner) runSamplesInner.addEventListener("click", runSamples);
    if (addSampleButton) {
      addSampleButton.addEventListener("click", () => {
        samples.push({ input: "", expected: "" });
        renderSamples();
      });
    }
    source.addEventListener("input", changed);
    source.addEventListener("click", inspect);
    source.addEventListener("keyup", inspect);
    editor.closest("form")?.addEventListener("reset", () => {
      queueMicrotask(() => {
        changed();
        stdout.textContent = "Run a program to see its output.";
      });
    });
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
        samples = (example.samples ?? []).map(s => ({ input: s.input, expected: s.expected }));
        renderSamples();
        stdout.textContent = "Run a program to see its output.";
        changed();
      };
      picker.addEventListener("change", select);
      const shared = decodeShare(location.hash);
      if (shared) {
        source.value = shared.source;
        stdin.value = shared.stdin;
        samples = shared.samples;
        renderSamples();
        stdout.textContent = "Loaded from shared link · click Run program or Run samples.";
        changed();
      } else {
        select();
      }
    }
  });
})();
