#!/usr/bin/env python3
"""Offline judge for the held-out acceptance problems (PROTOCOL.md, `solve` and `compete`).

    research/acceptance/judge.py cf2285a -- ./a.out        judge one solution
    research/acceptance/judge.py cf2285a --self            check the judge itself: brute vs samples

A problem directory `problems/<id>/` has `tests/*.in` + `.ans` (the statement samples),
`gen.py SEED` (a small random multitest input) and `brute.py` (an exhaustive solver for
those sizes). `check.py IN OUT` replaces the answer comparison where answers are not
unique. `max.py` writes one input at the statement's maximum constraints and `tl` holds
the statement's time limit in seconds. Verdict AC needs every sample and 200 generated
cases to agree, and the max input to finish within 2x `tl` (the slack covers this
machine against the judge's).
"""
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent / "problems"
CASES = 200


def run(cmd, inp, timeout=10):
    r = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError(f"exit {r.returncode}: {r.stderr.strip()[:300]}")
    return r.stdout


def same(out, ans):
    return out.lower().split() == ans.lower().split()


def judge(pid, cmd):
    d = HERE / pid
    py = [sys.executable]
    check = d / "check.py"

    def ok(inp, out, ans):
        if check.exists():
            with tempfile.TemporaryDirectory() as tmp:
                fi, fo = Path(tmp, "in"), Path(tmp, "out")
                fi.write_text(inp)
                fo.write_text(out)
                return subprocess.run(py + [str(check), str(fi), str(fo)], capture_output=True).returncode == 0
        return same(out, ans)

    cases = [(p.name, p.read_text(), p.with_suffix(".ans").read_text()) for p in sorted((d / "tests").glob("*.in"))]
    for name, inp, ans in cases:
        out = run(cmd, inp)
        if not ok(inp, out, ans):
            return f"WA on sample {name}"
    for seed in range(1, CASES + 1):
        inp = run(py + [str(d / "gen.py"), str(seed)], "")
        ans = run(py + [str(d / "brute.py")], inp, timeout=60)
        try:
            out = run(cmd, inp)
        except subprocess.TimeoutExpired:
            return f"TLE on gen seed {seed}"
        if not ok(inp, out, ans):
            return f"WA on gen seed {seed}\n--- input\n{inp}--- expected\n{ans}--- got\n{out}"
    note = ""
    if (d / "max.py").exists() and cmd[0] != sys.executable:
        import time
        big = run(py + [str(d / "max.py")], "")
        limit = 2 * float((d / "tl").read_text())
        t0 = time.perf_counter()
        try:
            run(cmd, big, timeout=limit)
        except subprocess.TimeoutExpired:
            return f"TLE on max input (> {limit:g} s)"
        note = f", max input {time.perf_counter() - t0:.2f} s"
    return f"AC ({len(cases)} samples, {CASES} generated{note})"


def main(argv):
    pid = argv[0].lower()
    if argv[1:] == ["--self"]:
        print(judge(pid, [sys.executable, str(HERE / pid / "brute.py")]).replace("generated", "generated, brute vs itself"))
        return 0
    v = judge(pid, argv[argv.index("--") + 1:])
    print(f"{pid}: {v}")
    return 0 if v.startswith("AC") else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
