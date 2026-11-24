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
  no-build <id> -            the port's C++ doesn't build
  traps <id> --max           on the max input the port's checked C++ (`cpl
                             build --checks all`: the interpreter's checks on
                             ranges, indexes and overflow) exits non-zero
  over-limit <id> --max      on the max input the port's judge C++ takes longer
                             than the port's `// limits:` time (default 2000
                             ms) or more memory than its limit (default 256 MB)
  slow <id> --max            on the max input the port's C++ takes over 1.5x
                             the accepted C++ (best of three runs, ignoring
                             times under 0.1 s; AGENTS.md goal 3)
  disagrees <id> <args>      the port's C++ and the accepted C++ (orig.cpp, or
                             the first accepted submission) give different
                             answers, judged by checker.py when there is one

`gen.py --max` prints one input at every bound the statement states at once
(the sums of a multitest too), shaped to be slowest for the obvious solution
(problem-packages/R6). The interpreter runs the seeded inputs; the max input
belongs to the C++ (problem-packages/R7), where the checked build stands in
for the interpreter's checks.

Interactive problems are skipped: gen.py prints the interactor's hidden
data, not the port's input.

Known findings live in corpus/generators-known.txt, which only shrinks.
Same flags as corpus/differential.py: --check, --update, --cpl, -j.
"""
import argparse, concurrent.futures, hashlib, json, os, re, resource, subprocess, sys, tempfile, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import audit, differential

ROOT = differential.ROOT
KNOWN = os.path.join(ROOT, "corpus", "generators-known.txt")
SEEDS = (1, 2, 3)
SIZES = (5, 50)
TIMEOUT = 20
SLOWER = 1.5
FLOOR = 0.1
RANGE = re.compile(rb"error\[R05\d\d\]|outside its range")
LIMITS = re.compile(r"^//\s*limits:\s*(\d+)\s*ms,\s*(\d+)\s*MB", re.M)
MAX_GEN_TIMEOUT = 120
CHECKED_SLOWER = 10


def limits(port):
    """The port's `// limits: T ms, M MB` line (what `cpl bench` reads), else the judge default."""
    m = LIMITS.search(open(port).read())
    return (int(m.group(1)), int(m.group(2))) if m else (2000, 256)


def unlimited_stack():
    """The judges' stack: Codeforces gives the whole memory limit, so deep recursion at the max is legal."""
    hard = resource.getrlimit(resource.RLIMIT_STACK)[1]
    resource.setrlimit(resource.RLIMIT_STACK, (hard, hard))


def measure(exe, data, timeout):
    """(exit code, stdout, seconds, peak MB) of one run, or None on a timeout."""
    with tempfile.TemporaryFile() as inp, tempfile.TemporaryFile() as out:
        inp.write(data)
        inp.seek(0)
        start = time.perf_counter()
        p = subprocess.Popen([exe], stdin=inp, stdout=out, stderr=subprocess.DEVNULL, preexec_fn=unlimited_stack)
        deadline = start + timeout
        while True:
            pid, status, usage = os.wait4(p.pid, os.WNOHANG)
            if pid:
                break
            if time.perf_counter() > deadline:
                p.kill()
                os.wait4(p.pid, 0)
                return None
            time.sleep(0.005)
        took = time.perf_counter() - start
        p.returncode = os.waitstatus_to_exitcode(status)
        out.seek(0)
        return p.returncode, out.read(), took, usage.ru_maxrss / 1024


def checked(pid, port, cpl):
    """The port's C++ with `--checks all`, built with g++ -O2 and cached by source; None if a step fails."""
    cache = os.path.join(ROOT, "target", "corpus-generators")
    os.makedirs(cache, exist_ok=True)
    cpp = os.path.join(cache, f"{pid}.{os.getpid()}.{threading.get_ident()}.cpp")
    try:
        if subprocess.run([cpl, "build", "--judge", "codeforces", "--checks", "all", port, "-o", cpp], capture_output=True).returncode:
            return None
        exe = os.path.join(cache, hashlib.sha256(open(cpp, "rb").read()).hexdigest()[:16])
        if not os.path.exists(exe):
            tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}.tmp"
            if subprocess.run(["g++", "-std=c++17", "-O2", "-o", tmp, cpp], capture_output=True).returncode:
                return None
            os.replace(tmp, exe)
        return exe
    finally:
        if os.path.exists(cpp):
            os.remove(cpp)


def gen(g, args, timeout=TIMEOUT):
    try:
        r = subprocess.run([sys.executable, g, *map(str, args)], capture_output=True, timeout=timeout)
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
    if not exe:
        found.append(f"no-build {pid} -")
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
    data = gen(g, ("--max",), MAX_GEN_TIMEOUT)
    if data is None:
        return found + [f"gen-fails {pid} --max"]
    return found + at_max(pid, d, port, cpl, exe, orig, data)


def at_max(pid, d, port, cpl, exe, orig, data):
    """The findings of the max input: it passes the checked C++, fits the limits, matches the accepted C++."""
    ms, mb = limits(port)
    found = []
    safe = checked(pid, port, cpl)
    if not safe:
        found.append(f"no-build {pid} -")
    else:
        r = measure(safe, data, max(TIMEOUT, CHECKED_SLOWER * ms / 1000))
        if r is None or r[0] != 0:
            found.append(f"traps {pid} --max")
    if not exe:
        return found
    runs = [measure(exe, data, 2 * ms / 1000 + 1) for _ in range(3)]
    if any(r is None or r[0] != 0 for r in runs):
        return found + [f"over-limit {pid} --max" if any(r is None for r in runs) else f"traps {pid} --max"]
    if min(r[2] for r in runs) > ms / 1000 or min(r[3] for r in runs) > mb:
        found.append(f"over-limit {pid} --max")
    if orig:
        wants = [measure(orig, data, 4 * ms / 1000 + 1) for _ in range(3)]
        if all(w is not None and w[0] == 0 for w in wants):
            if not agrees(d, data, runs[0][1], wants[0][1]):
                found.append(f"disagrees {pid} --max")
            mine, theirs = min(r[2] for r in runs), min(w[2] for w in wants)
            if mine >= FLOOR and mine > SLOWER * max(theirs, FLOOR):
                found.append(f"slow {pid} --max")
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
