#!/usr/bin/env python3
"""Playground parity tests: share links, sample runner, and WASM-vs-CLI error text."""

from __future__ import annotations

import json
from pathlib import Path
import re
import shutil
import os
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
CPL = REPO / "target/debug/cpl"


def normalize_output(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.replace("\r\n", "\n").strip().splitlines())


class TestPlaygroundParity(unittest.TestCase):
    def test_playground_html_has_share_samples_and_cli_stderr(self):
        page = (ROOT / "_build/playground.html").read_text()
        for attr in ("data-share", "data-run-samples", "data-samples-panel", "data-cli-stderr"):
            self.assertIn(attr, page, f"playground.html missing {attr}")
        m = re.search(
            r'<script id="playground-examples" type="application/json">(.*?)</script>',
            page,
            re.S,
        )
        self.assertIsNotNone(m, "playground.html missing #playground-examples JSON")
        examples = json.loads(m.group(1).replace("\\/", "/"))
        self.assertGreaterEqual(len(examples), 20)
        for ex in examples:
            self.assertIn("samples", ex, f"preset {ex['id']} missing samples")
            self.assertGreaterEqual(len(ex["samples"]), 1, f"preset {ex['id']} has 0 samples")

    def test_playground_preset_samples_pass_cli(self):
        page = (ROOT / "_build/playground.html").read_text()
        m = re.search(
            r'<script id="playground-examples" type="application/json">(.*?)</script>',
            page,
            re.S,
        )
        examples = json.loads(m.group(1).replace("\\/", "/"))
        with tempfile.TemporaryDirectory(prefix="cpl-pg-samples-") as tmp:
            for ex in examples[:8]:
                cp = Path(tmp) / f"{ex['id']}.cp"
                cp.write_text(ex["source"])
                for idx, sample in enumerate(ex["samples"], 1):
                    r = subprocess.run(
                        [str(CPL), "run", str(cp)],
                        input=sample["input"],
                        cwd=REPO,
                        capture_output=True,
                        text=True,
                        timeout=15,
                    )
                    self.assertEqual(r.returncode, 0, f"{ex['id']} sample {idx} failed: {r.stderr}")
                    self.assertEqual(
                        normalize_output(r.stdout),
                        normalize_output(sample["expected"]),
                        f"{ex['id']} sample {idx} output mismatch",
                    )

    def test_share_link_and_wasm_cli_error_parity_in_node(self):
        node = shutil.which("node")
        if not node:
            self.skipTest("node not installed")
        wasm_js = REPO / "dist/wasm/cpl.js"
        if not wasm_js.is_file():
            self.skipTest("dist/wasm/cpl.js not built")

        # 1. Verify share-link encode/decode round-trip using playground.js's exact logic.
        pg_js = (ROOT / "assets/playground.js").read_text()
        js_share_test = f"""
const window = {{}};
const document = {{ getElementById: () => null, querySelectorAll: () => [] }};
{pg_js}
const payload = {{
  source: "let n: Int = read()\\n// café 🚀\\nprint(n * 2)\\n",
  stdin: "21\\n",
  samples: [{{ input: "21\\n", expected: "42" }}, {{ input: "0\\n", expected: "0" }}],
}};
const hash = "#share=" + window.CpPlaygroundShare.encode(payload);
if (!/^[#a-zA-Z0-9=_-]+$/.test(hash)) throw new Error("non-url-safe share hash: " + hash);
const decoded = window.CpPlaygroundShare.decode(hash);
if (JSON.stringify(decoded) !== JSON.stringify(payload)) {{
  throw new Error("round-trip mismatch: " + JSON.stringify(decoded));
}}
if (window.CpPlaygroundShare.decode("#share=!!!invalid!!!") !== null) {{
  throw new Error("invalid share hash should return null");
}}
console.log("SHARE_OK");
"""
        r = subprocess.run([node, "-e", js_share_test], capture_output=True, text=True, timeout=15)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("SHARE_OK", r.stdout)

        # 2. Verify compiled WASM module matches `cpl` CLI stderr byte-for-byte on errors & traps.
        cases = [
            ("check", "e0201.cp", "print(unknown_var)\n", ""),
            ("check", "e0203.cp", "let x = 3\nprint(x.to_str())\n", ""),
            ("check", "e0303.cp", "let k = read(Int)\nprint(k)\n", ""),
            ("run", "r0001_bounds.cp", "let a: List<Int> = [1, 2]\nlet i: Int = read()\nprint(a[i])\n", "9\n"),
            ("run", "r0001_div0.cp", "let (a, b): (Int, Int) = read()\nprint(a / b)\n", "5 0\n"),
        ]
        with tempfile.TemporaryDirectory(prefix="cpl-wasm-parity-") as tmp:
            cli_expected = []
            for mode, fname, src, inp in cases:
                p = Path(tmp) / fname
                p.write_text(src)
                cmd = [str(CPL), mode, str(p)]
                res = subprocess.run(cmd, input=inp if mode == "run" else None, cwd=REPO, capture_output=True, text=True, timeout=15)
                # Normalize filename to `main.cp` because WASM uses `main.cp`.
                cli_expected.append({
                    "mode": mode,
                    "src": src,
                    "inp": inp,
                    "code": res.returncode,
                    "stderr": res.stderr.replace(str(p), "main.cp"),
                })

            wasm_script = f"""
import {{ load }} from {json.dumps(wasm_js.as_uri())};
const cpl = await load();
const cases = {json.dumps(cli_expected)};
for (const c of cases) {{
  const res = c.mode === "check" ? cpl.check(c.src) : cpl.run(c.src, c.inp);
  if (res.exitCode !== c.code) {{
    throw new Error(c.mode + " exitCode mismatch on " + JSON.stringify(c.src) + ": wasm=" + res.exitCode + " cli=" + c.code + "\\n--- WASM ---\\n" + res.stderr + "\\n--- CLI ---\\n" + c.stderr);
  }}
  if (res.stderr !== c.stderr) {{
    throw new Error(c.mode + " stderr mismatch:\\n--- WASM ---\\n" + res.stderr + "\\n--- CLI ---\\n" + c.stderr);
  }}
}}
console.log("WASM_PARITY_OK:" + cases.length);
"""
            r2 = subprocess.run(
                [node, "--input-type=module", "-e", wasm_script],
                capture_output=True,
                text=True,
                timeout=30,
            )
            self.assertEqual(r2.returncode, 0, r2.stderr)
            self.assertIn(f"WASM_PARITY_OK:{len(cases)}", r2.stdout)


if __name__ == "__main__":
    unittest.main()
