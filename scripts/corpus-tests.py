#!/usr/bin/env python3
"""Generate the corpus acceptance tests (`tests/NN.in` and `NN.ans`).

`make` writes `corpus/problems/<id>/tests/NN.in` and `NN.ans` for every
problem with a `spec.cp`: inputs from `corpus/problems/<id>/gen.py` or
`bench/*` SPECS, answers from the problem's accepted C++ submission.

Running corpus acceptance tests now lives in `crates/cpl-conform`:
    cargo run -p cpl-conform -- corpus [filter]

Usage:
    python3 scripts/corpus-tests.py make [ids...] [--cases 8] [--out DIR]
    python3 scripts/corpus-tests.py verify [ids...]

`--out DIR` writes `DIR/<id>/NN.in` and `NN.ans` instead of the problem's
own `tests/`, so a fresh set can be compared with the committed one.

`verify` reruns the accepted C++ on every committed `tests/*.in` and counts
the answers it reproduces byte for byte: the evidence a committed answer is
derivable (decisions.md 278).
"""
import argparse, hashlib, inspect, json, os, random, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROBLEMS = os.path.join(ROOT, "corpus", "problems")
CACHE = os.path.join(ROOT, "target", "corpus-cxx")


def problems(ids):
    have = sorted(p for p in os.listdir(PROBLEMS) if os.path.exists(os.path.join(PROBLEMS, p, "spec.cp")))
    return [p for p in have if not ids or p in ids]


def problem_gens(wanted):
    """Per-problem `corpus/problems/<id>/gen.py` scripts."""
    gens = {}
    for p in wanted:
        path = os.path.join(PROBLEMS, p, "gen.py")
        if os.path.isfile(path):
            gens[p] = ("script", path)
    return gens


def bench_gens(wanted):
    gens = {}
    sys.path.insert(0, os.path.join(ROOT, "bench"))
    for name in ("rust_max_n_audit", "corpus_generators"):
        try:
            mod = __import__(name)
        except Exception as e:
            print(f"skip bench/{name}: {e!r:.80}", file=sys.stderr)
            continue
        for row in mod.SPECS:
            p = os.path.basename(row[0])
            if p in wanted and p not in gens:
                gens[p] = ("bench", row[4])
    return gens


def generate(kind, f, seed, idx, tmp):
    if kind == "script":
        r = subprocess.run([sys.executable, f, str(seed), str(max(4, idx * 4))],
                           capture_output=True, text=True, timeout=20, check=True)
        out = r.stdout
        return out if out.endswith("\n") else out + "\n"
    if kind == "bench":
        path = os.path.join(tmp, "gen.in")
        f(path, False, {"run_orig": None})
        return open(path).read()
    params = list(inspect.signature(f).parameters)
    rng = random.Random(seed)
    args = []
    for name in params:
        if name in ("rng", "r", "rand"):
            args.append(rng)
        elif name in ("big", "medium", "large"):
            args.append(False)
        else:
            args.append(idx)
    random.seed(seed)
    out = f(*args)
    if isinstance(out, tuple):
        out = out[0]
    return out if out.endswith("\n") else out + "\n"


def accepted_cpp(p):
    meta = json.load(open(os.path.join(PROBLEMS, p, "meta.json")))
    for s in meta.get("submissions", []):
        if s.get("result") == "AC" and os.path.exists(os.path.join(ROOT, s["file"])):
            return os.path.join(ROOT, s["file"])
    return None


def binary(src):
    os.makedirs(CACHE, exist_ok=True)
    key = hashlib.sha1(open(src, "rb").read()).hexdigest()[:16]
    exe = os.path.join(CACHE, key)
    if not os.path.exists(exe):
        r = subprocess.run(["g++", "-std=c++17", "-O2", "-o", exe, src], capture_output=True, text=True)
        if r.returncode:
            raise RuntimeError("g++: " + r.stderr[:200])
    return exe


def make(args):
    ids = problems(args.ids)
    gens = problem_gens(ids)
    for p, g in bench_gens(ids).items():
        gens.setdefault(p, g)
    made, missing = 0, []
    with tempfile.TemporaryDirectory() as tmp:
        for p in ids:
            cpp = accepted_cpp(p)
            if p not in gens or not cpp:
                missing.append((p, "no generator" if p not in gens else "no accepted C++"))
                continue
            try:
                exe = binary(cpp)
                tests = os.path.join(args.out, p) if args.out else os.path.join(PROBLEMS, p, "tests")
                os.makedirs(tests, exist_ok=True)
                for i in range(1, args.cases + 1):
                    inp = generate(*gens[p], seed=1000 + i, idx=i, tmp=tmp)
                    r = subprocess.run([exe], input=inp, capture_output=True, text=True, timeout=20)
                    if r.returncode:
                        raise RuntimeError(f"accepted C++ exits {r.returncode} on case {i}")
                    open(os.path.join(tests, f"{i:02}.in"), "w").write(inp)
                    open(os.path.join(tests, f"{i:02}.ans"), "w").write(r.stdout)
                made += 1
            except Exception as e:
                missing.append((p, f"{e!r:.120}"))
    for p, why in missing:
        print(f"{p}: {why}")
    print(f"corpus-tests: tests for {made} of {len(ids)} problems")


def verify(args):
    total = {"same": 0, "differ": 0, "no accepted C++": 0}
    derivable = []
    for p in problems(args.ids):
        tests = os.path.join(PROBLEMS, p, "tests")
        ins = sorted(f for f in os.listdir(tests) if f.endswith(".in")) if os.path.isdir(tests) else []
        cpp = accepted_cpp(p) if ins else None
        if ins and not cpp:
            total["no accepted C++"] += len(ins)
            continue
        same = 0
        for f in ins:
            ans = os.path.join(tests, f[:-3] + ".ans")
            try:
                r = subprocess.run([binary(cpp)], stdin=open(os.path.join(tests, f), "rb"),
                                   capture_output=True, timeout=20)
                ok = os.path.exists(ans) and r.stdout == open(ans, "rb").read()
            except (RuntimeError, subprocess.TimeoutExpired):
                ok = False
            same += ok
            total["same" if ok else "differ"] += 1
        if ins and same == len(ins):
            derivable.append(p)
    print(f"corpus-tests: answers reproduced byte for byte: {total}")
    print(f"corpus-tests: {len(derivable)} problems with every answer derivable")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["make", "verify"])
    ap.add_argument("ids", nargs="*")
    ap.add_argument("--cases", type=int, default=8)
    ap.add_argument("--out")
    args = ap.parse_args()
    (verify if args.cmd == "verify" else make)(args)


if __name__ == "__main__":
    main()
