#!/usr/bin/env python3
"""Cheat-sheet parity: docs/one-page-cheat-sheet.md and the prototype compiler must not drift.

Three checks, run from proto/ (`python3 tests/test_cheatsheet_parity.py`):

1. Every case in tests/cheatsheet/*.cp builds with proto/cpc, compiles as C++17 under ASan + UBSan with
   -DLOCAL, and prints exactly its .out.
2. Every case in tests/cheatsheet/gaps/*.cp is a cheat-sheet construct the prototype does not implement yet.
   It must still fail (to build, to run, or to print its .out). A gap that starts passing fails this test
   until it is moved up into tests/cheatsheet/, so the ledger of gaps can only shrink knowingly.
3. Every content line of the cheat sheet is listed in tests/cheatsheet/COVERAGE.tsv with at least one case
   or gap. Editing the cheat sheet (a new construct, a changed rule) fails this test until the line is
   mapped again: `--update` rewrites the map, keeping existing rows and proposing cases for new lines.
"""
import hashlib, os, re, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.dirname(HERE)
CASES = os.path.join(HERE, "cheatsheet")
GAPS = os.path.join(CASES, "gaps")
SHEET = os.path.join(PROTO, "..", "docs", "one-page-cheat-sheet.md")
COVERAGE = os.path.join(CASES, "COVERAGE.tsv")
CXX = ["g++", "-std=c++17", "-O1", "-DLOCAL", "-fsanitize=address,undefined", "-fno-sanitize-recover=undefined"]

def run_case(path):
    """Build and run one .cp; returns None when it passes, else a one-line reason."""
    base = path[:-3]
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "t.cp")     # built from a copy: the build stamps its source
        open(src, "w").write(open(path).read())
        r = subprocess.run([sys.executable, os.path.join(PROTO, "cp.py"), "build", src], capture_output=True, text=True)
        if r.returncode or not os.path.exists(os.path.join(tmp, "t.cpp")):
            lines = [l for l in (r.stdout + r.stderr).splitlines() if l.strip() and not l.startswith("wrote")]
            return "build: " + (lines[-1] if lines else "failed")[:200]
        r = subprocess.run(CXX + ["-o", os.path.join(tmp, "t"), os.path.join(tmp, "t.cpp")], capture_output=True, text=True)
        if r.returncode:
            return "g++: " + next((l for l in r.stderr.splitlines() if "error" in l), "failed")[:200]
        inp = open(base + ".in").read() if os.path.exists(base + ".in") else ""
        try:
            r = subprocess.run([os.path.join(tmp, "t")], input=inp, capture_output=True, text=True, timeout=20)
        except subprocess.TimeoutExpired:
            return "run: timed out"
        if r.returncode:
            return f"run: exit {r.returncode}: {r.stderr.strip()[-200:]}"
        if os.path.exists(base + ".out") and r.stdout != open(base + ".out").read():
            return "output: " + r.stdout[:200].replace("\n", "|")
        if not os.path.exists(base + ".out") and os.path.dirname(path) == CASES:
            return "no .out file"
    return None

def names(d):
    if not os.path.isdir(d):
        return []
    return sorted(f[:-3] for f in os.listdir(d) if f.endswith(".cp"))

def sheet_lines():
    """(section, key, text) for every content line of the cheat sheet: bullets, prose and code lines."""
    out, sec = [], 0
    for raw in open(SHEET).read().splitlines():
        m = re.match(r"## (\d+)\.", raw)
        if m:
            sec = int(m.group(1))
            continue
        text = " ".join(raw.split())
        if not sec or not text or text in ("---", "```", "```cp") or text.startswith("#"):
            continue
        out.append((sec, hashlib.sha1(text.encode()).hexdigest()[:10], text))
    return out

def read_coverage():
    rows = {}
    if os.path.exists(COVERAGE):
        for l in open(COVERAGE).read().splitlines():
            if l and not l.startswith("#"):
                key, cases, _ = l.split("\t", 2)
                rows[key] = cases.split(",")
    return rows

def propose(sec, text, known):
    """Cases for a new cheat-sheet line: those of its section whose name shares a word with the line."""
    mine = [n for n in known if n.startswith(f"s{sec}_")]
    low = text.lower()
    hit = [n for n in mine if any(len(w) > 2 and w in low for w in n.split("_")[1:])]
    return hit or mine

def update_coverage():
    known = names(CASES) + names(GAPS)
    old = read_coverage()
    with open(COVERAGE, "w") as f:
        f.write("# cheat-sheet line (sha1 of its text)\tcases and gaps exercising it\tthe line\n")
        for sec, key, text in sheet_lines():
            cases = [c for c in old.get(key, []) if c in known] or propose(sec, text, known)
            f.write(f"{key}\t{','.join(cases)}\t{text[:110]}\n")

def main():
    if "--update" in sys.argv:
        update_coverage()
        print("COVERAGE.tsv rewritten; review the rows for the lines that changed")
        return 0
    failed = 0
    cases, gaps = [os.path.join(CASES, n + ".cp") for n in names(CASES)], [os.path.join(GAPS, n + ".cp") for n in names(GAPS)]
    with ThreadPoolExecutor(os.cpu_count() or 4) as pool:
        got = list(pool.map(run_case, cases + gaps))
    for path, why in zip(cases, got[:len(cases)]):
        if why:
            failed += 1
            print(f"PARITY FAILED: {os.path.relpath(path, PROTO)}: {why}")
    for path, why in zip(gaps, got[len(cases):]):
        if why is None:
            failed += 1
            print(f"GAP CLOSED: {os.path.relpath(path, PROTO)} now passes; move it (with a .out) to tests/cheatsheet/")
    known, rows = set(names(CASES)) | set(names(GAPS)), read_coverage()
    gap_names = set(names(GAPS))
    open_lines = 0
    for sec, key, text in sheet_lines():
        listed = rows.get(key)
        if not listed or not listed[0]:
            failed += 1
            print(f"CHEAT SHEET DRIFT: section {sec} line is not in COVERAGE.tsv (add a case, then --update): {text[:100]}")
            continue
        for n in listed:
            if n not in known:
                failed += 1
                print(f"COVERAGE.tsv names a missing case `{n}` for: {text[:80]}")
        open_lines += all(n in gap_names for n in listed)
    print(f"cheat-sheet parity: {len(cases)} cases, {len(gaps)} known gaps, "
          f"{len(sheet_lines())} cheat-sheet lines mapped ({open_lines} covered only by gaps)"
          + ("" if not failed else f"; {failed} FAILED"))
    return 1 if failed else 0

if __name__ == "__main__":
    sys.exit(main())
