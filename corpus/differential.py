#!/usr/bin/env python3
"""The interpreter against the emitted C++ on every judge-proven port's tests.

For each id in corpus/sets/v03.txt, `cpl run port.cp` and the C++ from
`cpl build --judge codeforces` (g++ -O2) run every test in the problem's
`tests/`, or in `target/corpus-tests/<id>/` once it has moved there (made
first when its tests.toml lists them). A
finding is `differ <id> <test>`: the two print different bytes or exit
differently (AGENTS.md goal 5: the interpreter and the C++ agree on every
program, including on input they reject); a new one names both exit codes and
the first stdout line that differs. A case the interpreter can't finish in
TIMEOUT seconds is skipped and counted, never a finding; a C++ timeout is
retried once, since the C++ outruns the interpreter that just finished. A
package with an interactor.py feeds neither side its case: the interactor
judges each, once per case, and a `differ` is the two verdicts disagreeing.

The same builds give two more findings: `over-32kb <id> -`, a Codeforces
build over the 32 KB every solution should fit (goal 5), and
`removed-spelling <file> -`, a port, dream or spec that `cpl migrate --check`
would rewrite (idiomatic-code/R8).

`cpl test` reads the same package (problem-packages/R8): `cpl-test-no-cases`
when it finds no cases there, `cpl-test-fails <id> <case>` when it rejects a
case, `cpl-test-unjudged` when an interactive package's cases go unjudged.

Every finding is a failure: there is no known list.

  python3 corpus/differential.py [ids...]   print findings
  --check                                   fail on any finding
  --cpl PATH                                default: target/release/cpl
"""
import argparse, concurrent.futures, glob, hashlib, json, os, subprocess, sys, threading

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "target", "corpus-differential")
TIMEOUT = 20
SKIPPED = []
TARGET_BYTES = 32 * 1024
BYTES = {}
COMPARED = []
DETAIL = {}


def tests(pid):
    for d in (os.path.join(ROOT, "corpus", "problems", pid, "tests"), os.path.join(ROOT, "target", "corpus-tests", pid)):
        found = sorted(glob.glob(os.path.join(d, "*.in")))
        if found:
            return found
    return []


class Out(tuple):
    """(exit code, stdout), compared as such; `err` keeps stderr for the report."""
    err = b""


def run(cmd, inp):
    try:
        with open(inp, "rb") as f:
            r = subprocess.run(cmd, stdin=f, capture_output=True, timeout=TIMEOUT)
        out = Out((r.returncode, r.stdout))
        out.err = r.stderr
        return out
    except subprocess.TimeoutExpired:
        return None


def native(pid, port, cpl):
    """The port's Codeforces C++ built with g++ -O2, cached by source (kept beside it as
    `<exe>.cpp`, which `cpl test` reads); None if either step fails."""
    os.makedirs(CACHE, exist_ok=True)
    cpp = os.path.join(CACHE, f"{pid}.{os.getpid()}.{threading.get_ident()}.cpp")
    if subprocess.run([cpl, "build", "--judge", "codeforces", port, "-o", cpp], capture_output=True).returncode:
        return None
    src = open(cpp, "rb").read()
    BYTES[pid] = len(src)
    key = hashlib.sha256(src).hexdigest()[:16]
    exe = os.path.join(CACHE, key)
    if not os.path.exists(exe):
        tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}.tmp"
        if subprocess.run(["g++", "-std=c++17", "-O2", "-o", tmp, cpp], capture_output=True).returncode:
            os.remove(cpp)
            return None
        os.replace(tmp, exe)
    os.replace(cpp, f"{exe}.cpp")
    return exe


def removed_spellings(ids, cpl):
    files = [f for p in ids for n in ("port.cp", "dream.cp", "spec.cp")
             if os.path.exists(f := os.path.join(ROOT, "corpus", "problems", p, n))]
    if not files:
        return []
    r = subprocess.run([cpl, "migrate", "--check", "--json", *files], capture_output=True, text=True)
    return sorted(f"removed-spelling {os.path.relpath(os.path.join(ROOT, f['path']), ROOT)} -"
                  for f in json.loads(r.stdout)["files"] if f["changed"] or f["diagnostics"])


def v03():
    return [l.split()[0] for l in open(os.path.join(ROOT, "corpus", "sets", "v03.txt")) if l.strip() and not l.startswith("#")]


def changed(base):
    """The v03 ports with a file under corpus/problems/<id>/ changed since `base` (`staged`: the index)."""
    cmd = ["git", "diff", "--name-only", *(["--cached"] if base == "staged" else [base]), "--", "corpus/problems"]
    files = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
    touched = {f.split("/")[2] for f in files if f.count("/") >= 3}
    return [p for p in v03() if p in touched]


def select(a):
    """The ports a run covers: the ids given, the ones changed since --changed, or all of v03."""
    if getattr(a, "update", False) and (a.changed or a.ids):
        sys.exit("--update rewrites the whole known list, so it runs over all of v03")
    return changed(a.changed) if a.changed else a.ids or v03()


def ratchet(name, path, found, a, what):
    """Print, check or update `found` against the shrink-only list at `path`; with no
    `path`, every finding is new."""
    known = {l.strip() for l in open(path) if l.strip() and not l.startswith("#")} if path and os.path.exists(path) else set()
    if path and a.update:
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
        print("  new: " + f + (f"  ({DETAIL[f]})" if f in DETAIL else ""), file=sys.stderr)
    return 1 if a.check and new else 0


def interactor(pid):
    """The package's interactor.py, which judges its cases instead of a byte compare."""
    path = os.path.join(ROOT, "corpus", "problems", pid, "interactor.py")
    return path if os.path.exists(path) else None


def make_cases(pid, cpl):
    subprocess.run([sys.executable, os.path.join(ROOT, "scripts", "corpus.py"), "make", pid],
                   cwd=ROOT, env={**os.environ, "CPL": cpl}, capture_output=True)


def made(pid, cpl):
    """The package's cases; a package listing them in tests.toml gets them made first."""
    ins = tests(pid)
    if not ins and os.path.exists(os.path.join(ROOT, "corpus", "problems", pid, "tests.toml")):
        make_cases(pid, cpl)
        ins = tests(pid)
    return ins


def check(pid, cpl):
    port = os.path.join(ROOT, "corpus", "problems", pid, "port.cp")
    if not os.path.exists(port):
        return []
    ins = made(pid, cpl)
    if not ins:
        return cpl_test(pid, port, cpl)
    exe = native(pid, port, cpl)
    if not exe:
        return [f"no-build {pid} -"]
    found = [f"over-32kb {pid} -"] if BYTES.get(pid, 0) > TARGET_BYTES else []
    judge = interactor(pid)
    if not judge:
        found += [f for inp in ins if (f := compare(pid, port, exe, inp, cpl))]
        return found + cpl_test(pid, port, cpl)
    ours = judged(cpl, port)
    return found + interacted(pid, exe, ins, judge, cpl, ours) + cpl_test(pid, port, cpl, ours)


def interacted(pid, exe, ins, judge, cpl, ours):
    """`differ <id> <test>` where interactor.py rules on the C++ unlike on the port.

    An interactive case is no stdin to feed both sides: the interactor talks
    to each, once per case, through `cpl test --interactor`."""
    if isinstance(ours, str):
        return []
    theirs = judged(cpl, f"{exe}.cpp", "--tests", os.path.dirname(ins[0]), "--interactor", judge)
    if isinstance(theirs, str):
        finding = f"differ {pid} -"
        DETAIL[finding] = f"c++: {theirs}"
        return [finding]
    port = {c["name"]: c for c in ours}
    cxx = {c["name"]: c for c in theirs}
    found = []
    for inp in ins:
        name = os.path.splitext(os.path.basename(inp))[0]
        a, b = port.get(name, {}), cxx.get(name, {})
        COMPARED.append(pid)
        if a.get("verdict") != b.get("verdict"):
            finding = f"differ {pid} {os.path.relpath(inp, ROOT)}"
            DETAIL[finding] = f"interp {a.get('verdict')} ({a.get('why')}), c++ {b.get('verdict')} ({b.get('why')})"
            found.append(finding)
    return found


def compare(pid, port, exe, inp, cpl):
    """`differ <id> <test>` when the interpreter and the C++ disagree on `inp`, else None.

    A C++ run that times out once is retried: the C++ runs far faster than the
    interpreter that just finished, so one timeout is a stalled runner, and only
    a second one is the C++ hanging."""
    interp = run([cpl, "run", port], inp)
    if interp is None:
        SKIPPED.append(f"{pid} {os.path.basename(inp)}")
        return None
    cxx = run([exe], inp)
    if cxx is None:
        cxx = run([exe], inp)
    COMPARED.append(pid)
    if cxx == interp:
        return None
    finding = f"differ {pid} {os.path.relpath(inp, ROOT)}"
    DETAIL[finding] = mismatch(interp, cxx)
    return finding


def mismatch(interp, cxx):
    """Which side did what: both exit codes, then the first stdout line that differs."""
    if cxx is None:
        return f"interp exit {interp[0]}, c++ timed out twice after {TIMEOUT} s"
    say = f"interp exit {interp[0]}, c++ exit {cxx[0]}"
    for side, out in (("interp", interp), ("c++", cxx)):
        err = b" | ".join(l.strip() for l in getattr(out, "err", b"").splitlines() if l.strip())
        if out[0] and err:
            say += f", {side} stderr {err[:400]!r}"
    a, b = interp[1].split(b"\n"), cxx[1].split(b"\n")
    for i, (x, y) in enumerate(zip(a, b)):
        if x != y:
            return f"{say}; stdout line {i + 1}: interp {x[:80]!r}, c++ {y[:80]!r}"
    if len(a) != len(b):
        return f"{say}; stdout: interp {len(a)} lines, c++ {len(b)}"
    return say


def judged(cpl, program, *args):
    """`cpl test <program> <args>`'s cases, or the finding when it gives none."""
    try:
        r = subprocess.run([cpl, "test", program, *args, "--format", "json"], cwd=ROOT, capture_output=True, text=True, timeout=10 * TIMEOUT)
    except subprocess.TimeoutExpired:
        return "cpl-test-timeout"
    if "has no cases" in r.stderr:
        return "cpl-test-no-cases"
    try:
        return json.loads(r.stdout)["cases"]
    except (ValueError, KeyError):
        return "cpl-test-error"


def cpl_test(pid, port, cpl, cases=None):
    """What `cpl test` makes of the package the corpus tools read (problem-packages/R8)."""
    cases = judged(cpl, port) if cases is None else cases
    if isinstance(cases, str):
        return [f"{cases} {pid} -"]
    found = [f"cpl-test-fails {pid} {c['name']}" for c in cases if c["verdict"] not in (None, "ok")]
    judges = os.path.join(os.path.dirname(port), "interactor.py")
    if os.path.exists(judges) and all(c["verdict"] is None for c in cases):
        found.append(f"cpl-test-unjudged {pid} -")
    return found


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "release", "cpl"))
    ap.add_argument("-j", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--changed", metavar="BASE", help="only the v03 ports changed since BASE (a commit, or `staged`)")
    ap.add_argument("ids", nargs="*")
    a = ap.parse_args(argv)
    ids = select(a)
    with concurrent.futures.ThreadPoolExecutor(a.j) as pool:
        found = sorted(f for fs in pool.map(lambda p: check(p, a.cpl), ids) for f in fs)
    found += removed_spellings(ids, a.cpl)
    return ratchet("differential", None, found, a, f"{len(ids)} ports, {len(COMPARED)} cases compared, {len(SKIPPED)} slow cases skipped")

if __name__ == "__main__":
    sys.exit(main())
