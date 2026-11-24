#!/usr/bin/env python3
"""The size of the C++ `cpl build --judge` writes for every v0.3 program,
against the 32 KB bar.

    python3 bench/sizes.py [--judge J] [--cpl PATH] [--report FILE] [--check]

Programs: corpus/problems/*/spec.cp, corpus/problems/*/port.cp,
research/dryrun/v03/*.cp, bench/*/*.cp and bench/zero-cost/*/*.cp. A
program that does not build is listed, not sized. --report writes one line
per program (bytes, the largest part, the path). --check fails when a built
program is over the bar but not in bench/oversize.txt, or is in it and fits,
and when any bench program does not build: nothing else builds the bench
programs, so a rename would rot them unseen. Enforces performance/R8.
"""
import argparse, glob, os, re, statistics, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BAR = 32 * 1024
WARN = 24 * 1024
OVERSIZE = os.path.join(ROOT, "bench", "oversize.txt")
SETS = (
    "corpus/problems/*/spec.cp",
    "corpus/problems/*/port.cp",
    "research/dryrun/v03/*.cp",
    "bench/*/*.cp",
    "bench/zero-cost/*/*.cp",
)


def programs():
    return [p for pat in SETS for p in sorted(glob.glob(pat, root_dir=ROOT))]


def listed(path):
    """The paths a list file names: the first word of each line, past comments."""
    with open(path) as f:
        return {(l.split("#")[0].split() or [""])[0] for l in f} - {""}


def oversize():
    return listed(OVERSIZE)


def build_verdict(broken):
    """What to say about the programs `broken` (path, why), and whether --check
    fails: every bench program must build; corpus programs are left to their suites."""
    bench = [(p, why) for p, why in broken if p.startswith("bench/")]
    return [f"sizes: {p} does not build ({why}); fix it" for p, why in bench], bool(bench)


def size(cpl, judge, path):
    """(bytes, the largest part) of the C++ for `path`, or (None, why)."""
    env = {k: v for k, v in os.environ.items() if k not in ("CPL_EDITION", "CPL_STD", "CPL_SPEC")}
    run = subprocess.run(
        [cpl, "build", path, "--judge", judge, "--size", "-o", os.devnull],
        cwd=ROOT, capture_output=True, text=True, timeout=120, env=env,
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
        note = " (listed in bench/oversize.txt)" if listed else ""
        messages.append(f"sizes: {p} is {n} bytes, over the {BAR}-byte bar{note}")
    for p in sorted(known - {p for _, p in over}):
        failed = True
        messages.append(f"sizes: {p} is not over the {BAR}-byte bar now: take it off bench/oversize.txt")
    return messages, failed


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--judge", default="codeforces")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("--report")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("-j", type=int, default=os.cpu_count())
    args = ap.parse_args()
    paths = programs()
    with ThreadPoolExecutor(args.j) as pool:
        got = list(pool.map(lambda p: size(args.cpl, args.judge, p), paths))
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
    more, broke = build_verdict(broken)
    messages, failed = messages + more, failed or broke
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
