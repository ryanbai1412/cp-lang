#!/usr/bin/env python3
"""Rejudge the committed acceptance-run solutions on the current compiler.

    research/acceptance/regress.py [cpl]              every run under solutions/
    research/acceptance/regress.py [cpl] --only cf2285d

`solutions/<run>/<os>/<id>.cp` are the files an agent submitted; `solutions/<run>/repro/
<id>-<what>.cp` are drafts that exposed a blocker (each says which in its first line).
Each is built with `cpl build --judge codeforces` and judged by judge.py, so a codegen or
stdlib change that makes an accepted run solution WA or TLE fails here. Exits 1 unless all
are AC.
"""
import shutil
import sys
import tempfile
from pathlib import Path

from score_solutions import HERE, ROOT, verdict


def main(argv):
    only = argv[argv.index("--only") + 1].lower() if "--only" in argv else None
    pos = [a for i, a in enumerate(argv) if not a.startswith("--") and (i == 0 or argv[i - 1] != "--only")]
    cpl = pos[0] if pos else shutil.which("cpl") or str(ROOT / "target" / "release" / "cpl")
    bad = total = 0
    with tempfile.TemporaryDirectory() as tmp:
        for src in sorted((HERE / "solutions").glob("*/*/*.cp")):
            pid = src.stem.split("-")[0].lower()
            if only and pid != only:
                continue
            v = verdict(cpl, src, pid, tmp, judge="codeforces")
            total += 1
            bad += not v.startswith("AC")
            print(f"{src.relative_to(HERE)}: {v}", flush=True)
    print(f"regress: {total - bad}/{total} AC")
    return 1 if bad or not total else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
