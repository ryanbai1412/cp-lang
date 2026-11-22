#!/usr/bin/env python3
"""The size of the C++ `cpl build --judge` writes for every corpus and
dry-run program, against the 32 KB bar (warn at 24 KB).

    python3 scripts/sizes.py [--judge J] [--cpl PATH] [--report FILE] [--check]

Programs: corpus/problems/*/port.cp and research/dryrun/*.cp as 0.2,
corpus/problems/*/spec.cp and research/dryrun/v03/*.cp as 0.3. A program that does not build is listed, not
sized; building is other suites' job. --report writes one line per program
(bytes, the largest part, the path). --check fails when a built program is
over the bar but not in corpus/sets/oversize.txt, or is in it and fits.
"""
import argparse, glob, os, re, statistics, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BAR = 32 * 1024
WARN = 24 * 1024
OVERSIZE = os.path.join(ROOT, "corpus", "sets", "oversize.txt")
# Each set in the edition it is written in (decisions.md 188).
V02 = {"CPL_EDITION": "0.2", "CPL_STD": os.path.join(ROOT, "stdlib")}
SETS = (
    ("corpus/problems/*/port.cp", V02),
    ("research/dryrun/*.cp", V02),
    ("corpus/problems/*/spec.cp", {}),
    ("research/dryrun/v03/*.cp", {}),
)


def programs():
    return [(p, env) for pat, env in SETS for p in sorted(glob.glob(pat, root_dir=ROOT))]


def oversize():
    with open(OVERSIZE) as f:
        return {l.split("#")[0].strip() for l in f} - {""}


def size(cpl, judge, path, edition):
    """(bytes, the largest part) of the C++ for `path`, or (None, why)."""
    env = {k: v for k, v in os.environ.items() if k not in ("CPL_EDITION", "CPL_STD", "CPL_SPEC")}
    run = subprocess.run(
        [cpl, "build", path, "--judge", judge, "--size", "-o", os.devnull],
        cwd=ROOT, capture_output=True, text=True, timeout=120, env=env | edition,
    )
    if run.returncode != 0:
        why = next((l for l in run.stderr.splitlines() if "error" in l), run.stderr.strip()[:200])
        return None, why
    total = re.search(r"size of the C\+\+ by part: (\d+) bytes", run.stderr)
    parts = re.findall(r"^  (\S.*?)\s+(\d+)\s+\d+%", run.stderr, re.M)
    biggest = max(parts, key=lambda p: int(p[1]), default=("?", "0"))
    return int(total.group(1)), f"{biggest[0]} {biggest[1]}"


def verdict(over, known):
    """What to say about the programs `over` the bar, as (bytes, path), given
    the paths `known` to be, and whether --check fails."""
    messages, failed = [], False
    for n, p in over:
        listed = p in known
        failed |= not listed
        note = " (listed in corpus/sets/oversize.txt)" if listed else ""
        messages.append(f"sizes: {p} is {n} bytes, over the {BAR}-byte bar{note}")
    for p in sorted(known - {p for _, p in over}):
        failed = True
        messages.append(f"sizes: {p} is not over the {BAR}-byte bar now: take it off corpus/sets/oversize.txt")
    return messages, failed


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--judge", default="codeforces")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("--report")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("-j", type=int, default=os.cpu_count())
    args = ap.parse_args()
    sets = programs()
    paths = [p for p, _ in sets]
    with ThreadPoolExecutor(args.j) as pool:
        got = list(pool.map(lambda s: size(args.cpl, args.judge, *s), sets))
    built = sorted(((n, part, p) for p, (n, part) in zip(paths, got) if n is not None), reverse=True)
    broken = [(p, why) for p, (n, why) in zip(paths, got) if n is None]
    lines = [f"{n}\t{part}\t{p}" for n, part, p in built] + [f"-\t{why}\t{p}" for p, why in broken]
    if args.report:
        with open(args.report, "w") as f:
            f.write("bytes\tlargest part\tprogram\n" + "\n".join(lines) + "\n")
    sizes = [n for n, _, _ in built]
    over = [(n, p) for n, _, p in built if n > BAR]
    warn = [(n, p) for n, _, p in built if WARN < n <= BAR]
    if sizes:
        print(f"{len(sizes)} programs built for judge `{args.judge}` ({len(broken)} do not build): "
              f"median {int(statistics.median(sizes))} bytes, max {sizes[0]} ({built[0][2]}); "
              f"{len(warn)} over {WARN}, {len(over)} over {BAR}")
    for n, p in warn:
        print(f"sizes: warning: {p} is {n} bytes, over the {WARN}-byte warning", file=sys.stderr)
    messages, failed = verdict(over, oversize())
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
