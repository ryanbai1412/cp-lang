#!/usr/bin/env python3
"""std is lint-clean (rule std-api/R21, handbook/guidelines/std-api-checklist.md#r21-std-is-lint-clean).

Runs `cpl lint --json std/` and fails on any hit: fix it in the std code.

    uv run python scripts/std-lint-ratchet.py --cpl target/debug/cpl
"""
import argparse, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def hits(cpl):
    env = dict(os.environ, CPL_STD_DIR=os.path.join(ROOT, "std"))
    out = subprocess.run([cpl, "lint", "--json", "std/"], cwd=ROOT, env=env,
                         capture_output=True, text=True).stdout
    return [(f["path"], d.get("code") or "?", d.get("message", ""))
            for f in json.loads(out)["files"] for d in f.get("diagnostics", [])]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    a = ap.parse_args()
    found = hits(a.cpl)
    for path, code, msg in found:
        print(f"{path}: {code}: {msg}")
    print(f"std lint: {len(found)} hit(s)" + ("; run `cpl lint std/` and fix each in the code" if found else ""))
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
