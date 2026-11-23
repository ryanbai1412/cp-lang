#!/usr/bin/env python3
"""Judge a run's solutions offline and write the verdicts into its JSON.

    research/acceptance/score_solutions.py runs/2026-11-23-1.json SOLUTIONS_DIR

For each problem in problems/div2.txt and problems/contest.txt, finds `<id>.cp` in
SOLUTIONS_DIR (any case), builds it with `cpl build`, compiles the C++ with
`g++ -std=c++17 -O2`, and runs judge.py. Sets each problem's `judge` field, and sets
`solve` / `compete` `pass` from the judge (>= 8/10 and >= 4/5), keeping the agent's own
call as `self_pass`.
"""
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent


def ids(name):
    return [l.split()[0] for l in (HERE / "problems" / name).read_text().split("\n") if l and not l.startswith("#")]


def verdict(cpl, src, pid, tmp, judge=None):
    cpp, exe = Path(tmp, pid + ".cpp"), Path(tmp, pid)
    r = subprocess.run([cpl, "build", *(["--judge", judge] if judge else []), str(src), "-o", str(cpp)], capture_output=True, text=True)
    if r.returncode != 0:
        return "CE (cpl build): " + (r.stdout + r.stderr).strip().split("\n")[0]
    r = subprocess.run(["g++", "-std=c++17", "-O2", str(cpp), "-o", str(exe)], capture_output=True, text=True)
    if r.returncode != 0:
        return "CE (g++): " + r.stderr.strip().split("\n")[0]
    r = subprocess.run([sys.executable, str(HERE / "judge.py"), pid.lower(), "--", str(exe)], capture_output=True, text=True)
    return (r.stdout.strip().split("\n")[0].split(": ", 1)[-1]) or r.stderr.strip()[:200]


def main(argv):
    run_path, sols = Path(argv[0]), Path(argv[1])
    run = json.loads(run_path.read_text())
    cpl = shutil.which("cpl") or str(ROOT / "target" / "release" / "cpl")
    files = {p.stem.lower(): p for p in sols.rglob("*.cp")}
    probs = {p["id"].lower(): p for p in run.setdefault("problems", [])}
    with tempfile.TemporaryDirectory() as tmp:
        for step, name, need in (("solve", "div2.txt", 8), ("compete", "contest.txt", 4)):
            ac = 0
            for pid in ids(name):
                p = probs.setdefault(pid.lower(), {"id": pid, "set": name[:-4], "verdict": "unsolved"})
                p.setdefault("id", pid)
                src = files.get(pid.lower())
                p["judge"] = verdict(cpl, src, pid, tmp) if src else "no solution"
                ac += p["judge"].startswith("AC")
                print(f"{pid}: {p['judge']}")
            s = run["steps"][step]
            s.setdefault("self_pass", s["pass"])
            s["pass"] = ac >= need
            s["judged"] = f"{ac}/{len(ids(name))} AC"
            print(f"{step}: {s['judged']} -> pass={s['pass']}")
    run["problems"] = list(probs.values())
    run_path.write_text(json.dumps(run, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
