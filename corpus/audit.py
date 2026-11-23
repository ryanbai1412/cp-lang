#!/usr/bin/env python3
"""Audit corpus problem packages against handbook/guidelines/problem-packages-checklist.md.

Each finding is `<rule> <problem> <detail>`:

  answer-is-input   an expected output equal to its input (more than two
                    tokens), so it was copied, not produced by a solution
                    (problem-packages/R2)
  needs-checker     two accepted C++ submissions print different outputs on
                    one test and the problem has no checker.py
                    (problem-packages/R3)
  interactor-no-gen an interactive problem with no gen.py for its hidden
                    inputs (problem-packages/R4)
  no-reference      a problem with tests and no accepted C++ with a `main`
                    (a grader-style submission), so its answers can't be
                    recomputed (problem-packages/R1)

A reference that needs a newer g++ than this machine's is skipped and
counted on stderr, never a finding, so findings don't depend on the
toolchain.

Known findings live in corpus/audit-known.txt, a ratchet that only shrinks.

  python3 corpus/audit.py            print findings
  python3 corpus/audit.py --check    fail on a finding not in the list
  python3 corpus/audit.py --update   rewrite the list from today's findings
  -j N                               parallel jobs (default: CPUs)
  --no-run                           skip checks that compile and run C++
"""
import argparse, concurrent.futures, glob, hashlib, json, os, re, subprocess, sys, threading

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROBLEMS = os.path.join(ROOT, "corpus", "problems")
KNOWN = os.path.join(ROOT, "corpus", "audit-known.txt")
CACHE = os.path.join(ROOT, "target", "corpus-audit")
TIMEOUT = 20
# The judges accept C++17 to C++23; the older spelling `gnu++2b` works on g++ 11.
STANDARDS = ("-std=c++17", "-std=gnu++2b")
MAIN = re.compile(r"\bmain\s*\(")
UNBUILT = []


def rel(p):
    return os.path.relpath(p, ROOT)


def cases(pdir):
    for inp in sorted(glob.glob(os.path.join(pdir, "tests", "*.in"))):
        for ext in (".out", ".ans"):
            ans = inp[:-3] + ext
            if os.path.exists(ans):
                yield inp, ans
                break


def tokens(path_or_bytes):
    data = path_or_bytes if isinstance(path_or_bytes, bytes) else open(path_or_bytes, "rb").read()
    return data.split()


def answer_is_input(pid, pdir):
    for inp, ans in cases(pdir):
        t = tokens(inp)
        if len(t) > 2 and t == tokens(ans):
            yield f"answer-is-input {pid} {rel(ans)}"


def is_interactive(pdir, meta):
    return os.path.exists(os.path.join(pdir, "interactor.py")) or "interactive" in json.dumps(meta.get("tags", [])).lower()


def interactor_no_gen(pid, pdir, meta):
    if is_interactive(pdir, meta) and not os.path.exists(os.path.join(pdir, "gen.py")):
        yield f"interactor-no-gen {pid} -"


def accepted_cpp(meta):
    out = [os.path.join(ROOT, f) for f in meta.get("artifacts", {}).get("original", [])
           if f.endswith(".cpp") and os.path.exists(os.path.join(ROOT, f))]
    for s in meta.get("submissions", []):
        f = s.get("file") or ""
        path = os.path.join(ROOT, f)
        if s.get("result") == "AC" and f.endswith(".cpp") and os.path.exists(path) and path not in out:
            out.append(path)
    return out


def binary(src):
    os.makedirs(CACHE, exist_ok=True)
    key = hashlib.sha256(open(src, "rb").read()).hexdigest()[:16]
    exe = os.path.join(CACHE, key)
    if os.path.exists(exe + ".fail"):
        return None
    if not os.path.exists(exe):
        tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}.tmp"
        for std in STANDARDS:
            r = subprocess.run(["g++", std, "-O2", "-o", tmp, src], capture_output=True)
            if r.returncode == 0:
                os.replace(tmp, exe)
                return exe
        open(exe + ".fail", "w").close()
        return None
    return exe


def run(exe, inp):
    try:
        with open(inp, "rb") as f:
            r = subprocess.run([exe], stdin=f, capture_output=True, timeout=TIMEOUT)
        return r.stdout.split() if r.returncode == 0 else None
    except subprocess.TimeoutExpired:
        return None


def references(pid, pdir, meta):
    """`no-reference` and `needs-checker`: both need the accepted C++ built."""
    tests = list(cases(pdir))
    if not tests or is_interactive(pdir, meta):
        return []
    srcs = accepted_cpp(meta)
    if not any(MAIN.search(open(s, errors="replace").read()) for s in srcs):
        return [f"no-reference {pid} -"]
    exes = [e for e in (binary(s) for s in srcs) if e]
    if len(exes) < len(srcs):
        UNBUILT.append(pid)
    if len(exes) < 2 or os.path.exists(os.path.join(pdir, "checker.py")):
        return []
    for inp, _ in tests:
        outs = [o for o in (run(e, inp) for e in exes) if o is not None]
        if len(set(map(tuple, outs))) > 1:
            return [f"needs-checker {pid} {rel(inp)}"]
    return []


def audit(pid, do_run):
    pdir = os.path.join(PROBLEMS, pid)
    try:
        meta = json.load(open(os.path.join(pdir, "meta.json")))
    except (OSError, ValueError):
        meta = {}
    found = list(answer_is_input(pid, pdir)) + list(interactor_no_gen(pid, pdir, meta))
    if do_run:
        found += references(pid, pdir, meta)
    return found


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--update", action="store_true")
    ap.add_argument("--no-run", action="store_true")
    ap.add_argument("-j", type=int, default=os.cpu_count() or 4)
    ap.add_argument("ids", nargs="*")
    a = ap.parse_args(argv)
    ids = a.ids or sorted(os.listdir(PROBLEMS))
    with concurrent.futures.ThreadPoolExecutor(a.j) as pool:
        found = sorted(f for fs in pool.map(lambda p: audit(p, not a.no_run), ids) for f in fs)
    known = set()
    if os.path.exists(KNOWN):
        known = {l.strip() for l in open(KNOWN) if l.strip() and not l.startswith("#")}
    if a.update:
        with open(KNOWN, "w") as f:
            f.write("# corpus/audit.py findings not yet fixed; this list only shrinks.\n")
            f.writelines(x + "\n" for x in found)
        print(f"audit: wrote {rel(KNOWN)} ({len(found)} findings)")
        return 0
    new = [f for f in found if f not in known]
    if not a.check:
        print("\n".join(found))
    fixed = len(known - set(found)) if not a.ids and not a.no_run else 0
    print(f"audit: {len(ids)} problems, {len(found)} findings, {len(new)} new, {fixed} fixed", file=sys.stderr)
    if UNBUILT:
        print(f"audit: {len(UNBUILT)} problems have a reference this g++ can't build; "
              f"needs-checker skipped some of their submissions", file=sys.stderr)
    if a.check and new:
        print("audit: new findings (see handbook/guidelines/problem-packages.md):", file=sys.stderr)
        for f in new:
            print("  " + f, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
