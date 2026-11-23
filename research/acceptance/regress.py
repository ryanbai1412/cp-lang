#!/usr/bin/env python3
"""Rejudge the committed acceptance-run solutions on the current compiler.

    research/acceptance/regress.py [cpl] [-j N]       every run under solutions/
    research/acceptance/regress.py [cpl] --only cf2285d
    research/acceptance/regress.py [cpl] --shard 2/3     every third, from the second (CI)

`solutions/<run>/<os>/<id>.cp` are the files an agent submitted; `solutions/<run>/repro/
<id>-<what>.cp` are drafts that exposed a blocker (each says which in its first line).
Each is built with `cpl build --judge codeforces` and judged by judge.py, so a codegen or
stdlib change that makes an accepted run solution WA or TLE fails here. Exits 1 unless all
are AC.

`-j N` (default: the CPU count) judges N solutions at once. Load only slows a run, so an
AC under load is an AC alone; a TLE under load is judged again by itself before it counts.
"""
import os
import shutil
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from score_solutions import HERE, ROOT, verdict


def opt(argv, name):
    return argv[argv.index(name) + 1] if name in argv else None


def main(argv):
    only, jobs = opt(argv, "--only"), int(opt(argv, "-j") or os.cpu_count() or 1)
    shard, shards = map(int, (opt(argv, "--shard") or "1/1").split("/"))
    pos = [a for i, a in enumerate(argv) if not a.startswith("-") and (i == 0 or argv[i - 1] not in ("--only", "-j", "--shard"))]
    cpl = pos[0] if pos else shutil.which("cpl") or str(ROOT / "target" / "release" / "cpl")
    srcs = [s for s in sorted((HERE / "solutions").glob("*/*/*.cp")) if not only or s.stem.split("-")[0].lower() == only.lower()]
    srcs = srcs[shard - 1::shards]
    with tempfile.TemporaryDirectory() as tmp:
        def judge(src):
            return verdict(cpl, src, src.stem.split("-")[0].lower(), tempfile.mkdtemp(dir=tmp), judge="codeforces")

        with ThreadPoolExecutor(max_workers=jobs) as pool:
            verdicts = list(pool.map(judge, srcs))
        for i, src in enumerate(srcs):
            if verdicts[i].startswith("TLE") and jobs > 1:
                verdicts[i] = judge(src) + " (rejudged alone after a TLE under -j)"
    bad = 0
    for src, v in zip(srcs, verdicts):
        bad += not v.startswith("AC")
        print(f"{src.relative_to(HERE)}: {v}")
    print(f"regress: {len(srcs) - bad}/{len(srcs)} AC")
    return 1 if bad or not srcs else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
