#!/usr/bin/env python3
"""Generate the corpus acceptance tests (`tests/NN.in` and `NN.ans`).

`make` writes `target/corpus-tests/<id>/NN.in` and `NN.ans` for every
problem with a `spec.cp`, or a `port.cp` and a `tests.toml`: inputs from the seed list in
`corpus/problems/<id>/tests.toml` (see `seed_list`), else from its `gen.py`
or `bench/*` SPECS, answers from the problem's accepted C++ submission. It
never writes into `corpus/`: committed test data is being retired
(handbook/guidelines/problem-packages-checklist.md#r9).

Running corpus acceptance tests now lives in `crates/cpl-conform`:
    cargo run -p cpl-conform -- corpus [filter]

Usage:
    python3 scripts/corpus-tests.py make [ids...] [--cases 8] [--out DIR]
    python3 scripts/corpus-tests.py verify [ids...] [--list]
    python3 scripts/corpus-tests.py seed ids...

`--out DIR` writes `DIR/<id>/NN.in` and `NN.ans` instead of
`target/corpus-tests/<id>/`.

`verify` reruns the accepted C++ on every committed `tests/*.in` and counts
the answers it reproduces byte for byte: the evidence a committed answer is
derivable (decisions.md 278).

A package with an `interactor.py` has no answers: `make` writes only the
hidden tests, and both commands judge the accepted C++ on them with
`cpl test --interactor` (`$CPL`, default `target/debug/cpl`), the same
judge `cpl test` and `cpl stress` use, so each case runs the interactor once.
"""
import argparse, hashlib, inspect, json, os, random, shutil, subprocess, sys, tempfile, tomllib

import corpus
from corpus import binary, judge_stack

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROBLEMS = os.path.join(ROOT, "corpus", "problems")
OUT = os.path.join(ROOT, "target", "corpus-tests")
INLINE = 200
CPL = os.environ.get("CPL", os.path.join(ROOT, "target", "debug", "cpl"))


def problems(ids):
    """Problems with a `spec.cp`, or a `port.cp` with a `tests.toml` seed list."""
    def has(p, f):
        return os.path.exists(os.path.join(PROBLEMS, p, f))
    have = sorted(p for p in os.listdir(PROBLEMS) if has(p, "spec.cp") or (has(p, "port.cp") and has(p, "tests.toml")))
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


def seed_list(p):
    """The cases in `corpus/problems/<id>/tests.toml`, or None without one.

    Each `[[case]]` is `seed` and `size` (gen.py's two arguments), `args` (a
    list, e.g. `["--max"]`), or a tiny hand-picked `input` written inline.
    """
    path = os.path.join(PROBLEMS, p, "tests.toml")
    if not os.path.exists(path):
        return None
    return tomllib.load(open(path, "rb")).get("case", [])


def case_input(p, case, timeout=60):
    if "input" in case:
        return case["input"] if case["input"].endswith("\n") else case["input"] + "\n"
    argv = [str(a) for a in case["args"]] if "args" in case else [str(case["seed"]), str(case["size"])]
    gen = os.path.join(PROBLEMS, p, "gen.py")
    out = subprocess.run([sys.executable, gen, *argv], capture_output=True, text=True, timeout=timeout, check=True).stdout
    return out if out.endswith("\n") else out + "\n"


def inputs(p, gens, n, tmp):
    """The problem's inputs: its seed list when it has one, else n cases from its generator."""
    listed = seed_list(p)
    if listed is not None:
        return [case_input(p, c) for c in listed]
    return [generate(*gens[p], seed=1000 + i, idx=i, tmp=tmp) for i in range(1, n + 1)]


def accepted_cpp(p):
    """The accepted C++ the package's answers come from (scripts/corpus.py)."""
    ref = corpus.Package.at(os.path.join(PROBLEMS, p)).reference()
    return str(ref) if ref else None


def interactor(p):
    path = os.path.join(PROBLEMS, p, "interactor.py")
    return path if os.path.exists(path) else None


def judged(p, cpp, tests):
    """The verdict of each case in `tests` for the accepted C++, judged by the package's interactor."""
    r = subprocess.run([CPL, "test", cpp, "--tests", tests, "--interactor", interactor(p), "--format", "json"],
                       capture_output=True, text=True, timeout=600)
    try:
        cases = json.loads(r.stdout)["cases"]
    except (ValueError, KeyError):
        raise RuntimeError(f"cpl test: {r.stderr.strip()[:200]}")
    return {c["name"]: c["verdict"] for c in cases}


def stamp(p, cpp):
    """What a problem's generated cases depend on; `make` skips a problem whose stamp is unchanged."""
    h = hashlib.sha256()
    for f in [os.path.join(PROBLEMS, p, "tests.toml"), os.path.join(PROBLEMS, p, "gen.py"), cpp]:
        if f and os.path.exists(f):
            h.update(open(f, "rb").read())
    return h.hexdigest()


def make(args):
    ids = problems(args.ids)
    if args.seeded:
        ids = [p for p in ids if seed_list(p) is not None]
    gens = problem_gens(ids)
    for p, g in bench_gens(ids).items():
        gens.setdefault(p, g)
    made, missing = 0, []
    with tempfile.TemporaryDirectory() as tmp:
        for p in ids:
            cpp = accepted_cpp(p)
            if (p not in gens and seed_list(p) is None) or not cpp:
                missing.append((p, "no generator" if p not in gens else "no accepted C++"))
                continue
            try:
                tests = os.path.join(args.out or OUT, p)
                key = os.path.join(tests, ".stamp")
                if seed_list(p) is not None and os.path.exists(key) and open(key).read() == stamp(p, cpp):
                    made += 1
                    continue
                exe = None if interactor(p) else binary(cpp)
                shutil.rmtree(tests, ignore_errors=True)
                os.makedirs(tests, exist_ok=True)
                for i, inp in enumerate(inputs(p, gens, args.cases, tmp), 1):
                    if interactor(p):
                        open(os.path.join(tests, f"{i:02}.in"), "w").write(inp)
                        continue
                    r = subprocess.run([exe], input=inp, capture_output=True, text=True, timeout=20, preexec_fn=judge_stack)
                    if r.returncode:
                        raise RuntimeError(f"accepted C++ exits {r.returncode} on case {i}")
                    open(os.path.join(tests, f"{i:02}.in"), "w").write(inp)
                    open(os.path.join(tests, f"{i:02}.ans"), "w").write(r.stdout)
                if interactor(p):
                    rejected = [n for n, v in judged(p, cpp, tests).items() if v != "ok"]
                    if rejected:
                        raise RuntimeError(f"the interactor rejects the accepted C++ on {', '.join(rejected)}")
                if seed_list(p) is not None:
                    open(key, "w").write(stamp(p, cpp))
                made += 1
            except Exception as e:
                missing.append((p, f"{e!r:.120}"))
    for p, why in missing:
        print(f"{p}: {why}")
    print(f"corpus-tests: tests for {made} of {len(ids)} problems")


def seed(args):
    """Write `tests.toml` for problems whose committed `tests/` it replaces.

    A committed input under INLINE bytes is kept inline; a larger one becomes a
    generated case of the largest power-of-two size whose input is no longer,
    so the seed list stays within the sizes the old files proved. `--max` is added when the
    generator has it.
    """
    for p in args.ids:
        d = os.path.join(PROBLEMS, p)
        gen = os.path.join(d, "gen.py")
        ins = sorted(f for f in os.listdir(os.path.join(d, "tests")) if f.endswith(".in"))
        lines = ["# Cases for scripts/corpus-tests.py make (problem-packages/R9).", ""]
        for n, f in enumerate(ins, 1):
            text = open(os.path.join(d, "tests", f)).read()
            if len(text) <= INLINE or not os.path.exists(gen):
                if len(text) > INLINE:
                    sys.exit(f"{p}: {f} is {len(text)} bytes and there is no gen.py")
                lines += ["[[case]]", "input = " + json.dumps(text), ""]
                continue
            size = 4
            while size < 1 << 24 and len(case_input(p, {"seed": n, "size": size * 2})) <= len(text):
                size *= 2
            lines += ["[[case]]", f"seed = {n}", f"size = {size}", ""]
        if os.path.exists(gen) and "--max" in open(gen).read():
            lines += ["[[case]]", 'args = ["--max"]', ""]
        open(os.path.join(d, "tests.toml"), "w").write("\n".join(lines).rstrip() + "\n")
        print(f"{p}: {len(ins)} committed cases -> tests.toml")


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
        if ins and interactor(p):
            try:
                verdicts = judged(p, cpp, tests)
            except (RuntimeError, subprocess.TimeoutExpired):
                verdicts = {}
            same = sum(v == "ok" for v in verdicts.values())
            total["same"] += same
            total["differ"] += len(verdicts) - same if verdicts else len(ins)
            if verdicts and same == len(verdicts):
                derivable.append(p)
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
    if args.list:
        print("\n".join(derivable))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["make", "verify", "seed"])
    ap.add_argument("ids", nargs="*")
    ap.add_argument("--cases", type=int, default=8)
    ap.add_argument("--out")
    ap.add_argument("--seeded", action="store_true", help="make: only problems with a tests.toml (fast once made)")
    ap.add_argument("--list", action="store_true", help="verify: print the problems whose answers all reproduce")
    args = ap.parse_args()
    {"make": make, "verify": verify, "seed": seed}[args.cmd](args)


if __name__ == "__main__":
    main()
