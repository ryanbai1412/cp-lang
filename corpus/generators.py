#!/usr/bin/env python3
"""Every judge-proven port's generator makes valid inputs, at the maximum too.

A port's `read<...>` types are the statement's constraints, so an input the
port rejects with a range error (R05xx in the interpreter, exit 3 in the
C++) breaks them: either gen.py is wrong or the port's types are. For each
id in corpus/sets/v03.txt with a gen.py, this runs `gen.py SEED SIZE` for
SEEDS x SIZES through `cpl run port.cp`, and `gen.py --max` through the
Codeforces C++ build. Findings (problem-packages/R5, R6):

  gen-fails <id> <args>      gen.py exits non-zero
  invalid-input <id> <args>  the port rejects what gen.py printed
  no-max <id> -              gen.py has no --max (no maximum-bound input)
  invalid-test <id> <file>   the port rejects a committed test input
  disagrees <id> <args>      the port's C++ and the accepted C++ (orig.cpp, or
                             the first accepted submission) give different
                             answers, judged by checker.py when there is one

Interactive problems are skipped: gen.py prints the interactor's hidden
data, not the port's input.

Known findings live in corpus/generators-known.txt, which only shrinks.
Same flags as corpus/differential.py: --check, --update, --cpl, -j.
"""
import argparse, concurrent.futures, json, os, re, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import audit, differential

ROOT = differential.ROOT
KNOWN = os.path.join(ROOT, "corpus", "generators-known.txt")
SEEDS = (1, 2, 3)
SIZES = (5, 50)
TIMEOUT = 20
RANGE = re.compile(rb"error\[R05\d\d\]|outside its range")


def gen(g, args):
    try:
        r = subprocess.run([sys.executable, g, *map(str, args)], capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return None
    return r.stdout if r.returncode == 0 else None


def rejects(cmd, data):
    try:
        r = subprocess.run(cmd, input=data, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return False
    return r.returncode != 0 and bool(RANGE.search(r.stderr))


def output(exe, data):
    try:
        r = subprocess.run([exe], input=data, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return None
    return r.stdout if r.returncode == 0 else None


def agrees(d, data, got, want):
    checker = os.path.join(d, "checker.py")
    if not os.path.exists(checker):
        return got.split() == want.split()
    with tempfile.TemporaryDirectory() as tmp:
        files = []
        for name, blob in (("in", data), ("out", got), ("ans", want)):
            files.append(os.path.join(tmp, name))
            open(files[-1], "wb").write(blob)
        return subprocess.run([sys.executable, checker, *files], capture_output=True).returncode == 0


def reference(d):
    try:
        meta = json.load(open(os.path.join(d, "meta.json")))
    except (OSError, ValueError):
        return None
    return next((e for e in map(audit.binary, audit.accepted_cpp(meta)) if e), None)


def check(pid, cpl):
    d = os.path.join(ROOT, "corpus", "problems", pid)
    g, port = os.path.join(d, "gen.py"), os.path.join(d, "port.cp")
    if not os.path.exists(port) or any(f.startswith("interactor") for f in os.listdir(d)):
        return []
    found = []
    for inp in differential.tests(pid):
        if rejects([cpl, "run", port], open(inp, "rb").read()):
            found.append(f"invalid-test {pid} {os.path.relpath(inp, ROOT)}")
    if not os.path.exists(g):
        return found
    exe, orig = differential.native(pid, port, cpl), reference(d)
    for seed in SEEDS:
        for size in SIZES:
            data = gen(g, (seed, size))
            if data is None:
                found.append(f"gen-fails {pid} {seed}/{size}")
            elif rejects([cpl, "run", port], data):
                found.append(f"invalid-input {pid} {seed}/{size}")
            elif exe and orig:
                got, want = output(exe, data), output(orig, data)
                if got is not None and want is not None and not agrees(d, data, got, want):
                    found.append(f"disagrees {pid} {seed}/{size}")
    if "--max" not in open(g).read():
        return found + [f"no-max {pid} -"]
    data = gen(g, ("--max",))
    if data is None:
        found.append(f"gen-fails {pid} --max")
    elif exe and rejects([exe], data):
        found.append(f"invalid-input {pid} --max")
    return found


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--update", action="store_true")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "release", "cpl"))
    ap.add_argument("-j", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--changed", metavar="BASE", help="only the v03 ports changed since BASE (a commit, or `staged`)")
    ap.add_argument("ids", nargs="*")
    a = ap.parse_args(argv)
    ids = differential.select(a)
    with concurrent.futures.ThreadPoolExecutor(a.j) as pool:
        found = sorted(f for fs in pool.map(lambda p: check(p, a.cpl), ids) for f in fs)
    return differential.ratchet("generators", KNOWN, found, a, f"{len(ids)} ports")


if __name__ == "__main__":
    sys.exit(main())
