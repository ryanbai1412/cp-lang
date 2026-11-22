#!/usr/bin/env python3
"""Make and run the corpus acceptance tests.

`make` writes `corpus/problems/<id>/tests/NN.in` and `NN.ans` for every
problem with a `spec.cp`: inputs from `corpus/problems/<id>/gen.py` or
`bench/*` SPECS, answers from the problem's accepted C++ submission.

`run` runs every `spec.cp` on its tests with `cpl run` and prints one
line per problem plus a summary; `--json` writes the per-problem result.

    python3 scripts/corpus-tests.py make [ids...] [--cases 8]
    python3 scripts/corpus-tests.py run [ids...] [--json out.json]
"""
import argparse, ast, glob, hashlib, inspect, json, os, random, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROBLEMS = os.path.join(ROOT, "corpus", "problems")
CPL = os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl"))
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
                tests = os.path.join(PROBLEMS, p, "tests")
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


def run(args):
    env = dict(os.environ, CPL_SPEC="1", CPL_STD=os.path.join(ROOT, "spec", "std"))
    results = {}
    for p in problems(args.ids):
        tests = sorted(glob.glob(os.path.join(PROBLEMS, p, "tests", "*.in")))
        if not tests:
            results[p] = {"status": "untested"}
            continue
        status, detail, passed = "pass", "", 0
        for t in tests:
            want = open(t[:-3] + ".ans").read().split()
            try:
                r = subprocess.run([CPL, "run", os.path.join(PROBLEMS, p, "spec.cp")], stdin=open(t),
                                   capture_output=True, text=True, timeout=30, env=env)
            except subprocess.TimeoutExpired:
                status, detail = "timeout", os.path.basename(t)
                break
            if r.returncode:
                err = (r.stderr or r.stdout).strip().splitlines()
                first = err[0] if err else f"exit {r.returncode}"
                status = "todo" if "`todo` reached" in first else ("error" if first.startswith("error[E") else "trap")
                detail = first[:160]
                break
            if r.stdout.split() != want:
                status, detail = "wrong", os.path.basename(t)
                break
            passed += 1
        results[p] = {"status": status, "passed": passed, "cases": len(tests), "detail": detail}
        print(f"{p:16} {status:8} {passed}/{len(tests)} {detail}")
    counts = {}
    for r in results.values():
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    print("corpus-tests:", ", ".join(f"{v} {k}" for k, v in sorted(counts.items())))
    if args.json:
        json.dump(results, open(args.json, "w"), indent=1, sort_keys=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["make", "run"])
    ap.add_argument("ids", nargs="*")
    ap.add_argument("--cases", type=int, default=8)
    ap.add_argument("--json")
    args = ap.parse_args()
    make(args) if args.cmd == "make" else run(args)


if __name__ == "__main__":
    main()
