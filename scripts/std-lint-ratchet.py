#!/usr/bin/env python3
"""std is lint-clean (rule std-api/R21, handbook/guidelines/std-api-checklist.md#r21-std-is-lint-clean).

Runs `cpl lint --json std/` and counts hits per (file, lint code). A count
above its line in scripts/std-lint-baseline.txt fails; a pair not in the
baseline allows 0. Fix a hit, or mark one that is right with
`with lint(name: allow)`, then lower the baseline with `--update`.

    uv run python scripts/std-lint-ratchet.py --cpl target/debug/cpl [--update]
"""
import argparse, collections, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASELINE = os.path.join(ROOT, "scripts", "std-lint-baseline.txt")


def counts(cpl):
    env = dict(os.environ, CPL_STD_DIR=os.path.join(ROOT, "std"))
    out = subprocess.run([cpl, "lint", "--json", "std/"], cwd=ROOT, env=env,
                         capture_output=True, text=True).stdout
    c = collections.Counter()
    for f in json.loads(out)["files"]:
        for d in f.get("diagnostics", []):
            c[(f["path"], d.get("code") or "?")] += 1
    return c


def load():
    base = {}
    for line in open(BASELINE):
        if line.strip() and not line.startswith("#"):
            path, code, n = line.split()
            base[(path, code)] = int(n)
    return base


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("--update", action="store_true", help="write today's counts as the baseline")
    a = ap.parse_args()
    now = counts(a.cpl)
    if a.update:
        with open(BASELINE, "w") as f:
            f.write("# std lint hits per file and code; only goes down (scripts/std-lint-ratchet.py)\n")
            for (path, code), n in sorted(now.items()):
                f.write(f"{path} {code} {n}\n")
        print(f"std lint: baseline written, {sum(now.values())} hit(s)")
        return 0
    base = load()
    over = [(k, n, base.get(k, 0)) for k, n in sorted(now.items()) if n > base.get(k, 0)]
    for (path, code), n, b in over:
        print(f"{path}: {code}: {n} hit(s), baseline {b}; run `cpl lint {path}` and fix the new one")
    lower = sum(1 for k, b in base.items() if now.get(k, 0) < b)
    print(f"std lint: {sum(now.values())} hit(s), baseline {sum(base.values())}"
          + (f"; {lower} count(s) went down, run with --update" if lower else ""))
    return 1 if over else 0


if __name__ == "__main__":
    sys.exit(main())
