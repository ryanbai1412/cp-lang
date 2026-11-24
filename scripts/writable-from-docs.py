#!/usr/bin/env python3
"""writable-from-docs.py: test whether a fresh model can solve unseen problems from docs alone.

Implements the AGENTS.md "Writable from the docs" design rule:
gives a fresh model ONLY the CP Lang docs bundle + unseen problem statements
from `corpus/statements/`, extracts each `.cp` program, runs `cpl check`,
`cpl build`, and `cpl run` on every sample case, records the pass rate and
every docs gap hit, and enforces a scoreboard ratchet (`research/writable-from-docs.json`).

Usage:
  python3 scripts/writable-from-docs.py --manifest research/writable-from-docs-set.json --workers 12
  python3 scripts/writable-from-docs.py --check-ratchet research/writable-from-docs.json
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime
import glob
import http.client
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

sys.path.insert(0, str(Path(__file__).parent))
from retry import retry  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
STATEMENTS_DIR = ROOT / "corpus/statements"
CPL = ROOT / "target/debug/cpl"
DEFAULT_MANIFEST = ROOT / "research/writable-from-docs-set.json"
DEFAULT_RATCHET = ROOT / "research/writable-from-docs.json"

DOC_FILES = [
    "web/content/docs/guides/cheatsheet.mdx",
    "web/content/docs/guides/tour.mdx",
    "web/content/docs/guides/types-and-control.mdx",
    "web/content/docs/guides/io-and-formatting.mdx",
    "web/content/docs/guides/indices.mdx",
    "web/content/docs/guides/borrowing.mdx",
    "web/content/docs/guides/strings.mdx",
    "web/content/docs/guides/stdlib.mdx",
    "web/content/docs/guides/cpp-guide.mdx",
    "web/content/docs/guides/idioms.mdx",
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
        Path(f).parent.name
        for pat in ("corpus/problems/*/spec.cp", "corpus/problems/*/port.cp")
        for f in glob.glob(str(root / pat))
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


def parse_openai_sse(lines) -> str:
    """Reassemble assistant text from an OpenAI chat-completions SSE stream."""
    chunks: list[str] = []
    for raw in lines:
        line = raw.decode("utf-8", errors="replace") if isinstance(raw, (bytes, bytearray)) else raw
        line = line.strip()
        if not line.startswith("data:"):
            continue
        payload = line[5:].strip()
        if not payload or payload == "[DONE]":
            continue
        try:
            obj = json.loads(payload)
        except json.JSONDecodeError:
            continue
        choices = obj.get("choices") or []
        if not choices:
            continue
        delta = choices[0].get("delta") or {}
        content = delta.get("content")
        if content:
            chunks.append(content)
    return "".join(chunks)


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
        with urllib.request.urlopen(req, timeout=180) as resp:
            data = json.loads(resp.read().decode())
        text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
        return mdl, text

    if openai_key:
        mdl = model or "gpt-5.4"
        payload: dict = {
            "model": mdl,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
        }
        if mdl.startswith(("gpt-5", "o3", "o4")):
            payload["max_completion_tokens"] = 24576
            payload["reasoning_effort"] = os.environ.get("WFD_REASONING_EFFORT", "medium")
            payload["stream"] = True
        else:
            payload["temperature"] = 0
            payload["max_tokens"] = 4096
        body = json.dumps(payload).encode()
        req = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=body,
            headers={
                "authorization": f"Bearer {openai_key}",
                "content-type": "application/json",
            },
            method="POST",
        )
        def once() -> tuple[str, str]:
            with urllib.request.urlopen(req, timeout=300) as resp:
                if payload.get("stream"):
                    return mdl, parse_openai_sse(resp)
                data = json.loads(resp.read().decode())
            return mdl, data["choices"][0]["message"]["content"]

        def transient(exc: Exception) -> bool:
            if isinstance(exc, urllib.error.HTTPError):
                return exc.code in (429, 500, 502, 503, 504)
            return isinstance(exc, (http.client.HTTPException, urllib.error.URLError, TimeoutError, OSError))

        return retry(once, transient=transient)

    raise RuntimeError("No ANTHROPIC_API_KEY or OPENAI_API_KEY in environment")


import os as _os, sys as _sys
_sys.path.insert(0, _os.path.join(_os.path.dirname(_os.path.abspath(__file__)), "..", "corpus", "lib"))
from judge_tokens import same_tokens  # noqa: E402  (cpl test's own comparison)


def outputs_match(got: str, want: str) -> bool:
    return same_tokens(want, got)


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
            if not outputs_match(got, want):
                result["error_summary"] = f"sample {idx} WA (expected {want[:40]!r}, got {got[:40]!r})"
                break
            passed_samples += 1

        result["samples_passed"] = passed_samples
        result["passed"] = result["check_ok"] and result["build_ok"] and (passed_samples == len(samples)) and len(samples) > 0

    result["seconds"] = round(time.monotonic() - t0, 2)
    return result


def summarize_results(results: list[dict], model_used: str) -> dict:
    total = len(results)
    compiled = sum(1 for r in results if r["check_ok"])
    built = sum(1 for r in results if r["build_ok"])
    passed = sum(1 for r in results if r["passed"])
    return {
        "date": datetime.date.today().isoformat(),
        "model": model_used,
        "total": total,
        "compiled": compiled,
        "built": built,
        "passed": passed,
        "compile_rate": round(compiled / total, 4) if total else 0.0,
        "pass_rate": round(passed / total, 4) if total else 0.0,
    }


def check_ratchet_data(ratchet: dict, summary: dict | None = None) -> list[str]:
    """Verify ratchet structure and (if summary given) no regression below floors."""
    errs: list[str] = []
    floor_c = ratchet.get("min_compile_rate")
    floor_p = ratchet.get("min_pass_rate")
    if not isinstance(floor_c, (int, float)) or not isinstance(floor_p, (int, float)):
        errs.append("ratchet missing numeric min_compile_rate / min_pass_rate")
        return errs
    cur = summary or ratchet.get("latest")
    if not cur:
        errs.append("ratchet missing latest metrics")
        return errs
    if cur["compile_rate"] + 1e-9 < floor_c:
        errs.append(
            f"compile_rate {cur['compile_rate']:.1%} dropped below ratchet floor {floor_c:.1%}"
        )
    if cur["pass_rate"] + 1e-9 < floor_p:
        errs.append(
            f"pass_rate {cur['pass_rate']:.1%} dropped below ratchet floor {floor_p:.1%}"
        )
    return errs


def format_markdown_report(results: list[dict], model_used: str) -> str:
    total = len(results)
    compiled = sum(1 for r in results if r["check_ok"])
    built = sum(1 for r in results if r["build_ok"])
    passed = sum(1 for r in results if r["passed"])
    lines = [
        "# Writable-from-docs audit",
        "",
        f"Model: `{model_used}` · Problems: **{total}** unseen statements from `corpus/statements/` (no existing port in `corpus/problems/`).",
        "",
        f"- **`cpl check` pass rate:** {compiled}/{total} ({100 * compiled / total:.1f}%)" if total else "- **No problems run**",
        f"- **`cpl build` pass rate:** {built}/{total} ({100 * built / total:.1f}%)" if total else "",
        f"- **Full pass (check + build + all samples):** {passed}/{total} ({100 * passed / total:.1f}%)" if total else "",
        "",
        "| Problem | `cpl check` | `cpl build` | Samples | Outcome / Error |",
        "| --- | --- | --- | --- | --- |",
    ]
    for r in results:
        chk = "PASS" if r["check_ok"] else "FAIL"
        bld = "PASS" if r["build_ok"] else "—"
        smp = f"{r['samples_passed']}/{r['samples_total']}"
        msg = "PASS" if r["passed"] else r["error_summary"].replace("|", "\\|")
        lines.append(f"| `{r['id']}` | {chk} | {bld} | {smp} | {msg} |")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Writable-from-docs harness for CP Lang")
    ap.add_argument("ids", nargs="*", help="Specific problem IDs from corpus/statements/")
    ap.add_argument("--manifest", type=Path, help="JSON manifest of problem IDs to evaluate")
    ap.add_argument("--limit", type=int, default=10, help="Max problems when auto-discovering")
    ap.add_argument("--workers", type=int, default=8, help="Concurrent model requests")
    ap.add_argument("--model", default=None, help="Override model name (default: gpt-5.4)")
    ap.add_argument("--report", type=Path, help="Write Markdown report to this path")
    ap.add_argument("--json-out", type=Path, help="Write raw JSON results to this path")
    ap.add_argument("--write-ratchet", type=Path, help="Update ratchet JSON (ratcheting floors upward)")
    ap.add_argument("--check-ratchet", type=Path, help="Validate ratchet JSON (offline or against this run)")
    ap.add_argument("--round-label", default="", help="Label for this round in ratchet history")
    args = ap.parse_args(argv)

    # Offline ratchet check mode (used by scoreboard.py --fast --check)
    if args.check_ratchet and not args.ids and not args.manifest and not args.write_ratchet and not args.json_out:
        ratchet = json.loads(args.check_ratchet.read_text())
        errs = check_ratchet_data(ratchet)
        if errs:
            for e in errs:
                print(f"writable-from-docs ratchet FAIL: {e}", file=sys.stderr)
            return 1
        cur = ratchet["latest"]
        print(
            f"writable-from-docs: {cur['compiled']}/{cur['total']} compile ({cur['compile_rate']:.1%}, "
            f"floor {ratchet['min_compile_rate']:.1%}), {cur['passed']}/{cur['total']} pass "
            f"({cur['pass_rate']:.1%}, floor {ratchet['min_pass_rate']:.1%}) [{cur['model']}]"
        )
        return 0

    if not CPL.is_file():
        print(f"error: {CPL} not built; run `cargo build -p cpl-cli`", file=sys.stderr)
        return 2

    if args.ids:
        targets = [STATEMENTS_DIR / f"{pid}.md" for pid in args.ids]
    elif args.manifest:
        data = json.loads(args.manifest.read_text())
        targets = [STATEMENTS_DIR / f"{pid}.md" for pid in data["problems"]]
    else:
        targets = discover_unported_statements()[: args.limit]

    targets = [p for p in targets if p.is_file()]
    if not targets:
        print("error: no unseen statements found in corpus/statements/", file=sys.stderr)
        return 2

    docs_bundle = load_docs_bundle()
    print(f"Loaded docs bundle ({len(docs_bundle):,} chars from {len(DOC_FILES)} files); evaluating {len(targets)} problems with {args.workers} workers...")

    results_by_id: dict[str, dict] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.workers)) as pool:
        futs = {pool.submit(evaluate_problem, p, docs_bundle, args.model): p for p in targets}
        for fut in concurrent.futures.as_completed(futs):
            r = fut.result()
            results_by_id[r["id"]] = r
            verdict = "PASS" if r["passed"] else f"FAIL ({r['error_summary']})"
            print(f"  {r['id']}: check={'ok' if r['check_ok'] else 'FAIL'} samples={r['samples_passed']}/{r['samples_total']} -> {verdict} [{r['seconds']}s]", flush=True)

    results = [results_by_id[p.stem] for p in targets]
    model_used = next((r["model"] for r in results if r["model"]), args.model or "unknown")
    summary = summarize_results(results, model_used)
    print(
        f"\nSummary ({model_used}): compile {summary['compiled']}/{summary['total']} "
        f"({summary['compile_rate']:.1%}), full pass {summary['passed']}/{summary['total']} "
        f"({summary['pass_rate']:.1%})"
    )

    if args.json_out:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(json.dumps({"summary": summary, "results": results}, indent=2) + "\n")
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(format_markdown_report(results, model_used))

    if args.write_ratchet:
        existing = json.loads(args.write_ratchet.read_text()) if args.write_ratchet.is_file() else {}
        history = existing.get("history", [])
        entry = dict(summary)
        if args.round_label:
            entry["round"] = args.round_label
        history.append(entry)
        # Ratchet floor: never goes down; allow a small 3% noise margin below best observed rate
        prev_c = existing.get("min_compile_rate", 0.0)
        prev_p = existing.get("min_pass_rate", 0.0)
        new_ratchet = {
            "manifest": "research/writable-from-docs-set.json",
            "model": model_used,
            "min_compile_rate": round(max(prev_c, max(0.0, summary["compile_rate"] - 0.05)), 4),
            "min_pass_rate": round(max(prev_p, max(0.0, summary["pass_rate"] - 0.05)), 4),
            "latest": summary,
            "history": history,
        }
        args.write_ratchet.parent.mkdir(parents=True, exist_ok=True)
        args.write_ratchet.write_text(json.dumps(new_ratchet, indent=2) + "\n")

    if args.check_ratchet:
        ratchet = json.loads(args.check_ratchet.read_text())
        errs = check_ratchet_data(ratchet, summary)
        if errs:
            for e in errs:
                print(f"RATCHET REGRESSION: {e}", file=sys.stderr)
            return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
