#!/usr/bin/env python3
"""The size of the C++ `cpl build --judge` writes for every v0.3 program,
against the 32 KB bar.

    python3 bench/sizes.py [--judge J] [--cpl PATH] [--report FILE] [--check]

Programs: every corpus package's port.cp, spec.cp and dream.cp (through
scripts/corpus.py's problems()), research/dryrun/v03/*.cp, bench/*/*.cp and
bench/zero-cost/*/*.cp. A
program that does not build is listed, not sized. --report writes one line
per program (bytes, the largest part, the path). --check fails when any built
program is over the bar, and when any bench program does not build: nothing else builds the bench
programs, so a rename would rot them unseen. Enforces performance/R8.
"""
import argparse, glob, os, re, statistics, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BAR = 32 * 1024
WARN = 24 * 1024
SETS = (
    "research/dryrun/v03/*.cp",
    "bench/*/*.cp",
    "bench/zero-cost/*/*.cp",
)
sys.path.insert(0, os.path.join(ROOT, "scripts"))
from corpus import problems  # noqa: E402


def programs():
    corpus = [
        os.path.relpath(f, ROOT)
        for kind in ("spec", "port", "dream")
        for pkg in problems()
        if (f := getattr(pkg, kind))
    ]
    return corpus + [p for pat in SETS for p in sorted(glob.glob(pat, root_dir=ROOT))]


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


def verdict(over):
    """What to say about the programs `over` the bar, as (bytes, path), and
    whether --check fails: any one over it does, as there is no list to excuse it."""
    return [f"sizes: {p} is {n} bytes, over the {BAR}-byte bar" for n, p in over], bool(over)


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
    messages, failed = verdict(over)
    more, broke = build_verdict(broken)
    messages, failed = messages + more, failed or broke
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
