#!/usr/bin/env python3
"""The interpreter against the emitted C++ on every judge-proven port's tests.

For each id in corpus/sets/v03.txt, `cpl run port.cp` and the C++ from
`cpl build --judge codeforces` (g++ -O2) run every test in the problem's
`tests/`, or in `target/corpus-tests/<id>/` once it has moved there. A
finding is `differ <id> <test>`: the two print different bytes or exit
differently (AGENTS.md goal 5: the interpreter and the C++ agree on every
program, including on input they reject). A case the interpreter can't
finish in TIMEOUT seconds is skipped and counted, never a finding.

Known findings live in corpus/differential-known.txt, which only shrinks.

  python3 corpus/differential.py [ids...]   print findings
  --check                                   fail on a finding not in the list
  --update                                  rewrite the list
  --cpl PATH                                default: target/release/cpl
"""
import argparse, concurrent.futures, glob, hashlib, os, subprocess, sys, threading

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KNOWN = os.path.join(ROOT, "corpus", "differential-known.txt")
CACHE = os.path.join(ROOT, "target", "corpus-differential")
TIMEOUT = 20
SKIPPED = []
COMPARED = []


def tests(pid):
    for d in (os.path.join(ROOT, "corpus", "problems", pid, "tests"), os.path.join(ROOT, "target", "corpus-tests", pid)):
        found = sorted(glob.glob(os.path.join(d, "*.in")))
        if found:
            return found
    return []


def run(cmd, inp):
    try:
        with open(inp, "rb") as f:
            r = subprocess.run(cmd, stdin=f, capture_output=True, timeout=TIMEOUT)
        return r.returncode, r.stdout
    except subprocess.TimeoutExpired:
        return None


def native(pid, port, cpl):
    """The port's Codeforces C++ built with g++ -O2, cached by source; None if either step fails."""
    os.makedirs(CACHE, exist_ok=True)
    cpp = os.path.join(CACHE, f"{pid}.{os.getpid()}.{threading.get_ident()}.cpp")
    if subprocess.run([cpl, "build", "--judge", "codeforces", port, "-o", cpp], capture_output=True).returncode:
        return None
    key = hashlib.sha256(open(cpp, "rb").read()).hexdigest()[:16]
    exe = os.path.join(CACHE, key)
    if not os.path.exists(exe):
        tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}.tmp"
        if subprocess.run(["g++", "-std=c++17", "-O2", "-o", tmp, cpp], capture_output=True).returncode:
            os.remove(cpp)
            return None
        os.replace(tmp, exe)
    os.remove(cpp)
    return exe


def v03():
    return [l.split()[0] for l in open(os.path.join(ROOT, "corpus", "sets", "v03.txt")) if l.strip() and not l.startswith("#")]


def ratchet(name, path, found, a, what):
    """Print, check or update `found` against the shrink-only list at `path`."""
    known = {l.strip() for l in open(path) if l.strip() and not l.startswith("#")} if os.path.exists(path) else set()
    if a.update:
        with open(path, "w") as f:
            f.write(f"# corpus/{name}.py findings not yet fixed; this list only shrinks.\n")
            f.writelines(x + "\n" for x in found)
        print(f"{name}: wrote {os.path.relpath(path, ROOT)} ({len(found)} findings)")
        return 0
    new = [f for f in found if f not in known]
    if not a.check:
        print("\n".join(found))
    print(f"{name}: {what}, {len(found)} findings, {len(new)} new, {len(known - set(found))} fixed", file=sys.stderr)
    for f in new if a.check else []:
        print("  new: " + f, file=sys.stderr)
    return 1 if a.check and new else 0


def check(pid, cpl):
    port = os.path.join(ROOT, "corpus", "problems", pid, "port.cp")
    ins = tests(pid)
    if not ins or not os.path.exists(port):
        return []
    exe = native(pid, port, cpl)
    if not exe:
        return [f"no-build {pid} -"]
    found = []
    for inp in ins:
        interp = run([cpl, "run", port], inp)
        if interp is None:
            SKIPPED.append(f"{pid} {os.path.basename(inp)}")
            continue
        cxx = run([exe], inp)
        COMPARED.append(pid)
        if cxx != interp:
            found.append(f"differ {pid} {os.path.relpath(inp, ROOT)}")
    return found


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--update", action="store_true")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "release", "cpl"))
    ap.add_argument("-j", type=int, default=os.cpu_count() or 4)
    ap.add_argument("ids", nargs="*")
    a = ap.parse_args(argv)
    ids = a.ids or v03()
    with concurrent.futures.ThreadPoolExecutor(a.j) as pool:
        found = sorted(f for fs in pool.map(lambda p: check(p, a.cpl), ids) for f in fs)
    return ratchet("differential", KNOWN, found, a, f"{len(ids)} ports, {len(COMPARED)} cases compared, {len(SKIPPED)} slow cases skipped")

if __name__ == "__main__":
    sys.exit(main())
