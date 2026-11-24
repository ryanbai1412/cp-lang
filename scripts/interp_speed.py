#!/usr/bin/env python3
"""Refuse a change that slows the interpreter past its base's own noise.

    interp_speed.py [--base REV] [--runs N] [--base-cpl PATH] [--tip-cpl PATH]

9a5881edd slowed the interpreter 57% and nothing refused it: the only timing gate
was the ratchet's interp_max_over_20s list, so a package over 20 s, or one well
under, could get any amount slower. This builds the base's release cpl in the
scratch base worktree with its own target dir (scripts/new_reds.py's) and the
tip's here, and runs every case of CASES, trimmed to a few seconds, in N rounds
of base, tip, base on this machine, timing each run's CPU seconds.

A case's noise floor is the largest ratio between two runs of the base, the max
over the min of its 2N base runs, computed from this run's own times; the case is
refused when its median tip run over its median base run is past that floor.
There is no fixed allowance: a quiet machine gets a tight floor, a loaded one a
wide floor, and a change that slows a case by more than the base differs from
itself is refused either way.
"""

import argparse
import os
import re
import resource
import statistics
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

PKGS = ROOT / "corpus/problems"
RUN_TIMEOUT = 120


def _lines(text: str) -> list[str]:
    return text.split("\n")


def wall(text: str) -> str:
    """ioi14p2's --max (2e6 columns, 5e5 operations) cut to 6000 columns and operations."""
    lines, n = _lines(text), 6000
    out = [f"{n} {n}"]
    for line in lines[1:1 + n]:
        t, l, r, h = line.split()
        l, r = sorted((int(l) % n, int(r) % n))
        out.append(f"{t} {l} {r} {h}")
    return "\n".join(out) + "\n"


def strip(text: str) -> str:
    """ioi99p6's --max (a 700 x 700 grid) cut to its top-left 150 x 150."""
    lines, s = _lines(text), 150
    c = lines[0].split()[2]
    return "\n".join([f"{s} {s} {c}", *(" ".join(row.split()[:s]) for row in lines[1:1 + s])]) + "\n"


def mushrooms(text: str) -> str:
    """ioi20p5's --max (20000 mushrooms) cut to its first N."""
    lines, n = _lines(text), MUSHROOMS
    return f"{n}\n{' '.join(lines[1].split()[:n])}\n"


MUSHROOMS = 2000


def lazy(source: str) -> str:
    """bench/ranges/lazy_assign.cp with n = q = 2e4 instead of 2e5 (42 s in the interpreter)."""
    for name in ("n", "q"):
        source, k = re.subn(rf"^let {name} = 200_000$", f"let {name} = 20_000", source, flags=re.M)
        if k != 1:
            raise SystemExit(f"interp_speed: bench/ranges/lazy_assign.cp no longer has `let {name} = 200_000`;"
                             " update CASES' trim")
    return source


@dataclass(frozen=True)
class Case:
    """A program run with its stdin; `trim` cuts the program (`source`) or its input
    (`max_input`, the package's gen.py --max) to a few seconds of interpretation."""
    name: str
    program: str
    max_input: bool = False
    trim: Callable[[str], str] | None = None
    source: bool = False


CASES = (
    Case("lazy_assign", "bench/ranges/lazy_assign.cp", trim=lazy, source=True),
    Case("std-ntt", "spec/conformance/std-ntt/convolve-past-an-ntt-primes-power-of-two.cp"),
    Case("ioi14p2", "corpus/problems/ioi14p2/port.cp", max_input=True, trim=wall),
    Case("ioi99p6", "corpus/problems/ioi99p6/port.cp", max_input=True, trim=strip),
    Case("ioi20p5", "corpus/problems/ioi20p5/port.cp", max_input=True, trim=mushrooms),
)


def prepare(case: Case, tmp: Path) -> tuple[Path, Path | None]:
    """(program, stdin) for `case` in `tmp`, trimmed."""
    program = ROOT / case.program
    if case.source:
        program = tmp / f"{case.name}.cp"
        program.write_text(case.trim(Path(ROOT / case.program).read_text()))
    if not case.max_input:
        return program, None
    gen = Path(case.program).parent / "gen.py"
    done = subprocess.run([sys.executable, str(ROOT / gen), "--max"], capture_output=True, text=True, check=True)
    stdin = tmp / f"{case.name}.in"
    stdin.write_text(case.trim(done.stdout) if case.trim else done.stdout)
    return program, stdin


def cpu_seconds(argv: list[str], stdin: Path | None) -> float:
    """The CPU seconds (user + sys) `argv` takes; a failed or timed-out run raises."""
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    with open(stdin or os.devnull, "rb") as f:
        done = subprocess.run(argv, stdin=f, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=RUN_TIMEOUT)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    if done.returncode:
        raise SystemExit(f"interp_speed: {' '.join(argv)} exited {done.returncode}:\n{done.stderr.decode()[-2000:]}")
    return (after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime)


@dataclass(frozen=True)
class Row:
    case: str
    base: float
    tip: float
    ratio: float
    floor: float

    @property
    def refused(self) -> bool:
        return self.ratio > self.floor


def measure(names, timer: Callable[[str, str], float], runs: int = 3) -> dict[str, tuple[list[float], list[float]]]:
    """{case: (base times, tip times)}: `runs` rounds of base, tip, base for each
    case, so drift over the run lands on both sides; `timer(side, case)` times one run."""
    out = {}
    for name in names:
        base, tip = [], []
        for _ in range(runs):
            base.append(timer("base", name))
            tip.append(timer("tip", name))
            base.append(timer("base", name))
        out[name] = (base, tip)
    return out


def judge(times: dict[str, tuple[list[float], list[float]]]) -> list[Row]:
    """A row per case: median tip over median base against the base's own spread."""
    rows = []
    for name, (base, tip) in times.items():
        b, t = statistics.median(base), statistics.median(tip)
        rows.append(Row(name, b, t, t / b, max(base) / min(base)))
    return rows


def report(rows: list[Row]) -> str:
    lines = [f"{'case':<12} {'base s':>8} {'tip s':>8} {'tip/base':>9} {'floor':>7}"]
    for r in rows:
        lines.append(f"{r.case:<12} {r.base:8.2f} {r.tip:8.2f} {r.ratio:9.3f} {r.floor:7.3f}"
                     + ("  REFUSED" if r.refused else ""))
    lines.append(f"noise floor (base vs base, computed): max {max(r.floor for r in rows):.3f},"
                 f" median {statistics.median(r.floor for r in rows):.3f}")
    return "\n".join(lines)


def profile() -> str:
    """The optimized profile: CI restores main's `ci`-profile target/ (scripts/ci.py)."""
    return "ci" if os.environ.get("CI") else "release"


def build(cwd: Path, target: Path, env=None) -> Path:
    p = profile()
    argv = ["cargo", "build", "-q", "--profile", p, "-p", "cpl-cli"]
    if subprocess.run(argv, cwd=cwd, env={**(env or os.environ), "CARGO_TARGET_DIR": str(target)}).returncode:
        raise SystemExit(f"interp_speed: {' '.join(argv)} failed in {cwd}")
    return target / p / "cpl"


def build_base(base: str) -> Path:
    import new_reds
    common = Path(new_reds.git("rev-parse", "--path-format=absolute", "--git-common-dir", cwd=ROOT))
    sha = new_reds.git("rev-parse", base, cwd=ROOT)
    wt = new_reds.base_checkout(common, sha)
    target = Path(os.environ.get("CPL_BASE_TARGET", str(common / "cpl-base-target")))
    return build(wt, target, new_reds.base_env())


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--base", default="HEAD", help="the base commit (default HEAD, which the commit tier stages against)")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--base-cpl", help="a built base cpl instead of building --base")
    ap.add_argument("--tip-cpl", help="a built tip cpl instead of building this tree")
    ap.add_argument("--only", nargs="*", help="just these cases")
    args = ap.parse_args(argv)
    cases = [c for c in CASES if not args.only or c.name in args.only]
    tip = Path(args.tip_cpl) if args.tip_cpl else build(ROOT, Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")))
    base = Path(args.base_cpl) if args.base_cpl else build_base(args.base)
    with tempfile.TemporaryDirectory(prefix="cpl-interp-speed-") as d:
        inputs = {c.name: prepare(c, Path(d)) for c in cases}
        cpl = {"base": str(base), "tip": str(tip)}

        def timer(side: str, name: str) -> float:
            program, stdin = inputs[name]
            return cpu_seconds([cpl[side], "run", str(program)], stdin)

        rows = judge(measure([c.name for c in cases], timer, args.runs))
    print(report(rows))
    refused = [r for r in rows if r.refused]
    for r in refused:
        print(f"interp_speed: {r.case} is {r.ratio:.2f}x its base in the interpreter, past the {r.floor:.3f}x"
              " the base differs from itself; find the slow path (cpl run --profile) and fix it", file=sys.stderr)
    return 1 if refused else 0


if __name__ == "__main__":
    sys.exit(main())
