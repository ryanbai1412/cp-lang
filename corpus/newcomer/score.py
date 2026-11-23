#!/usr/bin/env python3
"""Score the linter on the newcomer set: C++-style first drafts with marked spots.

    uv run python corpus/newcomer/score.py            summary + missing patterns by frequency
    uv run python corpus/newcomer/score.py --markdown the same as a Markdown table, for worklogs
    uv run python corpus/newcomer/score.py --fixes    also apply `cpl lint --fix` to a copy and
                                                      require `cpl check` to pass on the result

Every `.cp` here is a solution written the way a C++ contestant first writes CPL. Each
non-idiomatic spot carries `// expect: <lint name>` (a lint from `cpl lint --list`) or
`// expect: missing <pattern>` (no lint exists yet), or `a|b` when either lint teaches the spot, at the end of the line it is about.
A mark is flagged when that lint reports a diagnostic that covers the line or starts
within 3 lines of it (lints anchor on the accumulator, the loop or the fn); a `missing`
mark is flagged by any lint that does. Unmatched diagnostics are listed as unmarked.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
MARK = re.compile(r"//\s*expect:\s*(.+?)\s*$")


def cpl():
    for p in [os.environ.get("CPL"), ROOT / "target" / "release" / "cpl", ROOT / "target" / "debug" / "cpl", shutil.which("cpl")]:
        if p and Path(p).exists():
            return str(p)
    sys.exit("score: no cpl binary; build it with `cargo build -p cpl-cli`")


def lint_names(bin):
    out = subprocess.run([bin, "lint", "--list"], capture_output=True, text=True).stdout
    return {m.group(1): m.group(2) for m in re.finditer(r"^(L\d+)\s+(\S+)", out, re.M)}


def marks(path):
    found = []
    for n, line in enumerate(path.read_text().split("\n"), 1):
        m = MARK.search(line)
        if m:
            for want in m.group(1).split(","):
                found.append((n, want.strip()))
    return found


def diagnostics(bin, files, names):
    r = subprocess.run([bin, "lint", "--json", *map(str, files)], capture_output=True, text=True)
    data = json.loads(r.stdout or '{"files": []}')
    got = {}
    for f in data["files"]:
        for d in f["diagnostics"]:
            got.setdefault(Path(f["path"]).name, []).append((d["start"]["line"], names.get(d["code"], d["code"]), d))

    return got, data


def near(line, d):
    return d["start"]["line"] <= line <= d["end"]["line"] or abs(d["start"]["line"] - line) <= 3


def fix_failures(bin, files):
    bad = []
    with tempfile.TemporaryDirectory() as tmp:
        for f in files:
            c = Path(tmp) / f.name
            shutil.copy(f, c)
            subprocess.run([bin, "lint", "--fix", str(c)], capture_output=True, text=True)
            r = subprocess.run([bin, "check", str(c)], capture_output=True, text=True)
            if r.returncode != 0:
                bad.append((f.name, (r.stdout + r.stderr).strip().split("\n")[0]))
    return bad


def main(argv):
    bin = cpl()
    names = lint_names(bin)
    known = set(names.values())
    files = sorted(HERE.glob("*.cp"))
    broken = [f.name for f in files if subprocess.run([bin, "check", str(f)], capture_output=True).returncode != 0]
    got, _ = diagnostics(bin, files, names)
    total = hit = 0
    missed, unknown, fps = Counter(), [], Counter()
    for f in files:
        ms = marks(f)
        ds = got.get(f.name, [])
        used = set()
        for line, want in ms:
            total += 1
            if want.startswith("missing "):
                ok = any(near(line, d) for _, _, d in ds)
            else:
                wants = want.split("|")
                for w in wants:
                    if w not in known:
                        unknown.append(f"{f.name}:{line}: `{w}`")
                ok = any(n in wants and near(line, d) for _, n, d in ds)
            if ok:
                hit += 1
                used |= {i for i, (_, n, d) in enumerate(ds) if near(line, d) and (n in want.split("|") or want.startswith("missing "))}
            else:
                missed[want] += 1
        for i, (l, n, _) in enumerate(ds):
            if i not in used:
                fps[n] += 1
    pct = 100 * hit / total if total else 0
    lines = [f"newcomer: {hit}/{total} marked spots flagged ({pct:.0f}%), {len(files)} files, "
             f"{sum(fps.values())} unmarked diagnostics, {len(broken)} files fail `cpl check`"]
    if broken:
        lines.append("  fail cpl check: " + ", ".join(broken))
    if unknown:
        lines.append("  marks naming no lint: " + "; ".join(unknown))
    if "--fixes" in argv:
        bad = fix_failures(bin, files)
        lines.append(f"  fixes: {len(files) - len(bad)}/{len(files)} files still pass `cpl check` after `cpl lint --fix`")
        lines += [f"    {n}: {e}" for n, e in bad]
    if "--markdown" in argv:
        lines += ["", "| # | Missed pattern | Spots | Lint exists |", "|---|---|---|---|"]
        lines += [f"| {i} | `{p.removeprefix('missing ')}` | {c} | {'yes, did not fire' if not p.startswith('missing ') else 'no'} |"
                  for i, (p, c) in enumerate(missed.most_common(), 1)]
        if fps:
            lines += ["", "Unmarked diagnostics (possible false positives): " + ", ".join(f"`{n}` {c}" for n, c in fps.most_common())]
    else:
        lines += [f"  missed {c:3}  {p}" for p, c in missed.most_common()]
        if fps:
            lines.append("  unmarked: " + ", ".join(f"{n} {c}" for n, c in fps.most_common()))
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
