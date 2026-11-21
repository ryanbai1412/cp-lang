#!/usr/bin/env python3
"""Time to a counterexample for wrong solutions to every judged port.

For each ecnerwala port with an interactor or a checker, stress two kinds
of wrong solution under that judge with a fixed seed and case count:
`print(1)`, and up to one mutant per operator (an off-by-one, a flipped
comparison), the first site of each that still checks. Prints each run
and the p50/p95 of the time to the counterexample over the caught runs.

usage: bench/stress/judged.py [path/to/cpl]
"""
import json, os, re, shutil, statistics, subprocess, sys, tempfile, time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PORTS = os.path.join(ROOT, "proto", "ports", "ecnerwala")
CPL = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "target", "debug", "cpl")
NEEDS_ANSWER = {"cf2062g", "cf2187b"}
CASES = "1000"
OPERATORS = [
    ("+1→+2", r"\+ 1\b", "+ 2"),
    ("-1→-2", r"- 1\b", "- 2"),
    ("<→<=", r" < ", " <= "),
    ("<=→<", r" <= ", " < "),
    ("0..→1..", r"\b0\.\.", "1.."),
]


def judged():
    for sub, flag in [("interactors", "--interactor"), ("checkers", "--checker")]:
        for f in sorted(os.listdir(os.path.join(PORTS, sub))):
            if f.endswith(".py") and not f.startswith("_"):
                yield f[:-3], flag, os.path.join(PORTS, sub, f)


def checks(path):
    return subprocess.run([CPL, "check", path], capture_output=True).returncode == 0


def mutants(src, scratch, name):
    lines = src.split("\n")
    for label, pat, rep in OPERATORS:
        for at, line in enumerate(lines):
            code = line.split("#")[0]
            if code.lstrip().startswith("@") or not re.search(pat, code):
                continue
            mutated = lines[:]
            mutated[at] = re.sub(pat, rep, code, count=1) + line[len(code):]
            path = os.path.join(scratch, label.replace("→", "_").replace("<", "lt").replace("=", "e").replace("+", "p").replace(".", "d"), f"{name}.cp")
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w") as out:
                out.write("\n".join(mutated))
            if checks(path):
                yield f"{label} at line {at + 1}", path
                break


def stress(path, name, flag, judge, fast=None):
    cmd = [CPL, "stress", path, flag, judge, "--gen", os.path.join(PORTS, "gen", f"{name}.py"),
           "--seed", "1", "--cases", CASES, "--format", "json"]
    if name in NEEDS_ANSWER:
        cmd += ["--brute", os.path.join(PORTS, f"{name}_orig.cpp")]
    if fast:
        cmd += ["--fast", fast]
    start = time.monotonic()
    out = subprocess.run(cmd, capture_output=True, text=True)
    took = time.monotonic() - start
    try:
        said = json.loads(out.stdout)
    except ValueError:
        return took, "error", None
    case = (said.get("counterexample") or {}).get("case")
    return took, said.get("status"), case


def main():
    scratch = tempfile.mkdtemp(prefix="judged-")
    wrong = os.path.join(scratch, "wrong.py")
    with open(wrong, "w") as out:
        out.write("print(1)\n")
    caught, missed = [], []
    for name, flag, judge in judged():
        src = open(os.path.join(PORTS, f"{name}.cp")).read()
        runs = [("print(1)", None)]
        runs += list(mutants(src, os.path.join(scratch, name), name))
        for label, path in runs:
            here = os.path.join(scratch, name, "copy" if path is None else "", f"{name}.cp")
            if path is None:
                os.makedirs(os.path.dirname(here), exist_ok=True)
                shutil.copy(os.path.join(PORTS, f"{name}.cp"), here)
            took, status, case = stress(path or here, name, flag, judge, wrong if path is None else None)
            print(f"{name:<9} {flag[2:]:<10} {label:<24} {status:<6} case {case}  {took:6.2f} s", flush=True)
            (caught if status == "fail" else missed).append((name, label, took, status))
    for kind, pick in [("print(1)", lambda l: l == "print(1)"), ("mutants", lambda l: l != "print(1)")]:
        times = sorted(t for _, label, t, _ in caught if pick(label))
        lost = sum(1 for _, label, _, _ in missed if pick(label))
        q = statistics.quantiles(times, n=20, method="inclusive") if len(times) > 1 else times * 19
        print(f"{kind}: {len(times)} caught, {lost} not caught in {CASES} cases; time to "
              f"counterexample p50 {statistics.median(times):.2f} s, p95 {q[18]:.2f} s")
    shutil.rmtree(scratch, ignore_errors=True)


main()
