#!/usr/bin/env python3
"""writable-from-docs.py: test whether a fresh model can solve unseen problems from docs alone.

Implements the AGENTS.md "Writable from the docs" design rule:
gives a fresh model ONLY the CP Lang docs bundle + an unseen problem statement
from `corpus/statements/`, extracts its `.cp` program, runs `cpl check`,
`cpl run` on every sample case, and `cpl build`, and records the pass rate
and every docs gap hit.

Usage:
  python3 scripts/writable-from-docs.py                  # run on up to 10 unported statements
  python3 scripts/writable-from-docs.py --limit 10 --report research/writable-from-docs.md
  python3 scripts/writable-from-docs.py abc380b abc381c  # run on specific IDs
"""

from __future__ import annotations

import argparse
import concurrent.futures
import glob
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
STATEMENTS_DIR = ROOT / "corpus/statements"
CPL = ROOT / "target/debug/cpl"

DOC_FILES = [
    "docs/one-page-cheat-sheet.md",
    "site/content/tour.md",
    "site/content/types-and-control.md",
    "site/content/io-and-formatting.md",
    "site/content/indices.md",
    "site/content/borrowing.md",
    "site/content/strings.md",
    "site/content/stdlib.md",
]


def load_docs_bundle(root: Path = ROOT) -> str:
    parts = []
    for rel in DOC_FILES:
        p = root / rel
        if p.is_file():
            parts.append(f"=== {rel} ===\n{p.read_text().strip()}\n")
    return "\n".join(parts)


def ported_ids(root: Path = ROOT) -> set[str]:
    return {
        Path(f).stem
        for f in glob.glob(str(root / "proto/ports/**/*.cp"), recursive=True)
        if "/dream/" not in f.replace("\\", "/")
    }


def discover_unported_statements(root: Path = ROOT) -> list[Path]:
    ported = ported_ids(root)
    st_dir = root / "corpus/statements"
    if not st_dir.is_dir():
        return []
    return [p for p in sorted(st_dir.glob("*.md")) if p.stem not in ported]


def parse_samples(statement: str) -> list[tuple[str, str]]:
    """Extract (sample_input, sample_output) pairs from AtCoder or Codeforces markdown."""
    samples: list[tuple[str, str]] = []
    # AtCoder style: "Sample Input k\n```\n...\n```\n...\nSample Output k\n```\n...\n```"
    atcoder_in = {
        m.group(1): m.group(2).strip("\n") + "\n"
        for m in re.finditer(
            r"Sample Input\s+(\d+)\s*```[^\n]*\n(.*?)```", statement, re.S
        )
    }
    atcoder_out = {
        m.group(1): m.group(2).strip("\n")
        for m in re.finditer(
            r"Sample Output\s+(\d+)\s*```[^\n]*\n(.*?)```", statement, re.S
        )
    }
    if atcoder_in:
        for k in sorted(atcoder_in, key=int):
            if k in atcoder_out:
                samples.append((atcoder_in[k], atcoder_out[k]))
        return samples

    # Codeforces style: alternating Input / Output fenced blocks under Examples
    ex_idx = statement.find("Examples")
    chunk = statement[ex_idx:] if ex_idx >= 0 else statement
    blocks = re.findall(r"(Input|Output)\s*```[^\n]*\n(.*?)```", chunk, re.S)
    cur_in: str | None = None
    for kind, body in blocks:
        if kind == "Input":
            cur_in = body.strip("\n") + "\n"
        elif kind == "Output" and cur_in is not None:
            samples.append((cur_in, body.strip("\n")))
            cur_in = None
    return samples


def extract_cp_code(response_text: str) -> str | None:
    """Extract the first ```cp ... ``` (or fallback ``` ... ```) fenced block."""
    m = re.search(r"```(?:cp|cpl)[^\n]*\n(.*?)```", response_text, re.S | re.I)
    if m:
        return m.group(1).strip() + "\n"
    m = re.search(r"```[^\n]*\n(.*?)```", response_text, re.S)
    if m:
        return m.group(1).strip() + "\n"
    return None


def normalize_output(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.strip().splitlines())


def call_model(system_prompt: str, user_prompt: str, model: str | None = None) -> tuple[str, str]:
    """Call Anthropic or OpenAI with zero repo context; returns (model_used, text)."""
    anth_key = os.environ.get("ANTHROPIC_API_KEY")
    openai_key = os.environ.get("OPENAI_API_KEY")

    if anth_key and (model is None or model.startswith("claude")):
        mdl = model or "claude-sonnet-4-5-20250929"
        body = json.dumps(
            {
                "model": mdl,
                "max_tokens": 4096,
                "temperature": 0,
                "system": system_prompt,
                "messages": [{"role": "user", "content": user_prompt}],
            }
        ).encode()
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "x-api-key": anth_key,
                "anthropic-version": "2023-06-01",
                "content-type": "application/json",
            },
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = json.loads(resp.read().decode())
        text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
        return mdl, text

    if openai_key:
        mdl = model or "gpt-4o"
        body = json.dumps(
            {
                "model": mdl,
                "temperature": 0,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_prompt},
                ],
            }
        ).encode()
        req = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=body,
            headers={
                "authorization": f"Bearer {openai_key}",
                "content-type": "application/json",
            },
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = json.loads(resp.read().decode())
        return mdl, data["choices"][0]["message"]["content"]

    raise RuntimeError("No ANTHROPIC_API_KEY or OPENAI_API_KEY in environment")


def evaluate_problem(
    st_path: Path, docs_bundle: str, model: str | None = None
) -> dict:
    pid = st_path.stem
    statement = st_path.read_text()
    samples = parse_samples(statement)
    system_prompt = (
        "You are solving a competitive programming problem in CP Lang, using ONLY the "
        "CP Lang documentation provided below. Do not invent syntax or stdlib functions "
        "that are not in the documentation. Output one complete, runnable CP Lang solution "
        "inside a ```cp ... ``` code block, followed by a brief `DOCS_NOTES:` bullet list "
        "only if anything in the docs was unclear or missing.\n\n"
        + docs_bundle
    )
    user_prompt = f"Solve this problem in CP Lang:\n\n{statement}"

    t0 = time.monotonic()
    result: dict = {
        "id": pid,
        "samples_total": len(samples),
        "samples_passed": 0,
        "check_ok": False,
        "build_ok": False,
        "passed": False,
        "error_codes": [],
        "error_summary": "",
        "code": "",
        "model": "",
        "seconds": 0.0,
    }
    try:
        mdl, raw = call_model(system_prompt, user_prompt, model=model)
        result["model"] = mdl
    except Exception as exc:
        result["error_summary"] = f"API error: {exc}"
        result["seconds"] = round(time.monotonic() - t0, 2)
        return result

    code = extract_cp_code(raw)
    if not code:
        result["error_summary"] = "Model did not emit a fenced cp code block"
        result["seconds"] = round(time.monotonic() - t0, 2)
        return result
    result["code"] = code

    with tempfile.TemporaryDirectory(prefix=f"cpl-wfd-{pid}-") as tmp:
        cp_file = Path(tmp) / f"{pid}.cp"
        cp_file.write_text(code)

        chk = subprocess.run(
            [str(CPL), "check", str(cp_file)],
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=30,
        )
        if chk.returncode != 0:
            err = (chk.stdout + "\n" + chk.stderr).strip()
            result["error_codes"] = sorted(set(re.findall(r"\[([EWLR]\d{4})\]", err)))
            first_err = next((l for l in err.splitlines() if "error[" in l), err.splitlines()[0] if err else "check failed")
            result["error_summary"] = first_err
            result["full_error"] = err
            result["seconds"] = round(time.monotonic() - t0, 2)
            return result
        result["check_ok"] = True

        bld = subprocess.run(
            [str(CPL), "build", str(cp_file), "-o", str(Path(tmp) / f"{pid}.cpp")],
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=30,
        )
        result["build_ok"] = bld.returncode == 0
        if not result["build_ok"]:
            result["error_summary"] = (bld.stderr or bld.stdout).strip().splitlines()[0]

        passed_samples = 0
        for idx, (inp, exp) in enumerate(samples, 1):
            try:
                run = subprocess.run(
                    [str(CPL), "run", str(cp_file)],
                    input=inp,
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
            except subprocess.TimeoutExpired:
                result["error_summary"] = f"sample {idx} TLE (>15s)"
                break
            if run.returncode != 0:
                err = (run.stderr or run.stdout).strip()
                result["error_summary"] = f"sample {idx} runtime error: {err.splitlines()[0] if err else 'trap'}"
                result["full_error"] = err
                break
            got = normalize_output(run.stdout)
            want = normalize_output(exp)
            if got != want:
                result["error_summary"] = f"sample {idx} WA (expected {want[:40]!r}, got {got[:40]!r})"
                break
            passed_samples += 1

        result["samples_passed"] = passed_samples
        result["passed"] = result["check_ok"] and result["build_ok"] and (passed_samples == len(samples)) and len(samples) > 0

    result["seconds"] = round(time.monotonic() - t0, 2)
    return result


def format_markdown_report(results: list[dict], model_used: str) -> str:
    total = len(results)
    chk_ok = sum(1 for r in results if r["check_ok"])
    passed = sum(1 for r in results if r["passed"])
    lines = [
        "# Writable-from-Docs Audit",
        "",
        f"Model: `{model_used}` · Problems: {total} unseen statements from `corpus/statements/` (no existing port) · "
        f"Compile pass: **{chk_ok}/{total}** · Full sample + build pass: **{passed}/{total}**",
        "",
        "| Problem | Check | Build | Samples | Codes | Outcome / First error |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for r in results:
        chk = "ok" if r["check_ok"] else "fail"
        bld = "ok" if r["build_ok"] else ("—" if not r["check_ok"] else "fail")
        smp = f"{r['samples_passed']}/{r['samples_total']}"
        codes = ", ".join(r["error_codes"]) if r["error_codes"] else "—"
        summary = "PASS" if r["passed"] else r["error_summary"].replace("|", "\\|")[:100]
        lines.append(f"| `{r['id']}` | {chk} | {bld} | {smp} | {codes} | {summary} |")
    lines.append("")
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description="Run the writable-from-docs test on unseen statements.")
    ap.add_argument("ids", nargs="*", help="Specific problem IDs from corpus/statements/")
    ap.add_argument("--limit", type=int, default=10, help="Max problems when no IDs are given")
    ap.add_argument("--model", default=None, help="Override model ID")
    ap.add_argument("--jobs", type=int, default=4, help="Parallel API requests")
    ap.add_argument("--report", default=None, help="Write Markdown report to this path")
    ap.add_argument("--json", dest="json_out", default=None, help="Write raw JSON results to this path")
    args = ap.parse_args()

    if not CPL.is_file():
        sys.exit(f"writable-from-docs: {CPL} not built; run cargo build -p cpl-cli")

    if args.ids:
        paths = [STATEMENTS_DIR / f"{pid}.md" for pid in args.ids]
        missing = [p.name for p in paths if not p.is_file()]
        if missing:
            sys.exit(f"writable-from-docs: missing statements: {', '.join(missing)}")
    else:
        paths = discover_unported_statements()[: args.limit]
        if not paths:
            sys.exit("writable-from-docs: no unported statements in corpus/statements/; run scripts/fetch-statement.py")

    docs_bundle = load_docs_bundle()
    print(f"Docs bundle: {len(docs_bundle):,} chars across {len(DOC_FILES)} files; testing {len(paths)} unseen problems...")

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = list(pool.map(lambda p: evaluate_problem(p, docs_bundle, args.model), paths))

    model_used = next((r["model"] for r in results if r["model"]), args.model or "unknown")
    report = format_markdown_report(results, model_used)
    print("\n" + report)

    if args.report:
        Path(args.report).write_text(report)
        print(f"Wrote report → {args.report}")
    if args.json_out:
        Path(args.json_out).write_text(json.dumps(results, indent=2))
        print(f"Wrote JSON → {args.json_out}")


if __name__ == "__main__":
    main()
