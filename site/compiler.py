"""Package the real compiler once, for static pages and the offline edition."""

import base64
import gzip
import html
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent


def build_compiler(target):
    subprocess.run([str(ROOT.parent / "scripts/build-wasm.sh")], check=True)
    wasm = ROOT.parent / "dist/wasm"
    # Single dependency-free ES module -> classic Blob worker, including file://.
    bridge = re.sub(r"^export ", "", (wasm / "cpl.js").read_text(), flags=re.M)
    bridge = bridge.replace("import.meta.url", "self.location.href")
    worker = bridge + "\n" + (ROOT / "assets/compiler-worker.js").read_text()
    packed = base64.b64encode(gzip.compress((wasm / "cpl.wasm").read_bytes(), mtime=0)).decode()
    source = "(() => {\nconst workerSource = " + json.dumps(worker) + ";\n"
    source += "const packedWasm = " + json.dumps(packed) + ";\n"
    source += (ROOT / "assets/compiler.js").read_text() + "\n})();\n"
    (target / "assets/compiler.js").write_text(source)


def editor_html(key, source="", stdin=""):
    """Accessible editor shared by the playground and course sandboxes."""
    e = html.escape
    return f'''<div class="compiler-editor" data-cpl-editor>
      <div class="playground-toolbar"><span class="runner-label">CP Lang · WebAssembly</span>
        <div class="compiler-actions"><button class="secondary-button" data-build type="button">C++17</button>
        <button class="secondary-button" data-stop type="button" disabled>Stop</button>
        <button class="primary-button" data-run type="button">Run program</button></div></div>
      <div class="runner-grid">
        <div class="source-panel"><label for="{key}-source">Source · editable</label>
          <textarea class="source-editor" id="{key}-source" data-source spellcheck="false" autocapitalize="off" autocomplete="off" wrap="off" aria-describedby="{key}-hover">{e(source)}</textarea>
          <p class="compiler-hover" id="{key}-hover" data-hover role="status">Place the cursor on a name to inspect its type.</p></div>
        <div class="runner-io"><label for="{key}-stdin">Standard input</label>
          <textarea id="{key}-stdin" data-stdin spellcheck="false">{e(stdin)}</textarea>
          <label for="{key}-stdout">Standard output</label>
          <pre class="runner-output" id="{key}-stdout" data-stdout role="status">Run a program to see its output.</pre></div>
      </div>
      <p class="runner-status" data-status role="status">Ready · Ctrl / ⌘ + Enter to run</p>
      <div class="compiler-diagnostics" data-diagnostics aria-live="polite"></div>
      <details class="compiler-cpp" data-cpp-panel><summary>Generated C++17</summary><pre tabindex="0"><code data-cpp></code></pre></details>
    </div>'''
