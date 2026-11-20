#!/usr/bin/env python3
"""Line counts for every port, measured two ways (plan/80-validation.md).

raw       port lines against the accepted C++ submission
stripped  port lines against the submission minus what a perfect C++ contest library
          would have supplied; those line ranges are listed in ports/<name>.strip

Blank lines and comment-only lines never count, on either side. The whole port counts,
including its test generators and `where` preconditions, which the originals do not have;
those lines are also reported on their own.

usage: loc.py [--md] [dir]

dir is a directory of ports under ports/ (default: ports/ itself). If it has a before/
directory, the same ports as they had to be written before a language or library change,
those are measured too and the drop is reported.
"""
import glob, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
PORTS = f"{HERE}/ports"

def counted(path, block_comments):
    """1-based numbers of the lines that count"""
    out, inside = [], False
    for i, line in enumerate(open(path, errors="replace").read().split("\n"), 1):
        s = line.strip()
        if inside:
            inside = "*/" not in s
            continue
        if not s or s.startswith("//"):
            continue
        if block_comments and s.startswith("/*"):
            inside = "*/" not in s
            continue
        out.append(i)
    return out

def stripped_ranges(path):
    rows = []
    for line in open(path):
        if line.strip() and not line.startswith("#"):
            span, facility, note = line.split(None, 2)
            a, b = map(int, span.split("-"))
            rows.append((a, b, facility, note.strip()))
    return rows

def scaffolding(path):
    """lines of top-level fns that only serve tests: named in `read(gen: f)`, used in a
    `where` precondition, or called only by other such fns"""
    lines = open(path).read().split("\n")
    blocks, i = {}, 0
    while i < len(lines):
        m = re.match(r"(?:pure )?fn (\w+)", lines[i])
        j = i + 1
        if m:
            while j < len(lines) and (not lines[j].strip() or lines[j][0] in " \t"):
                j += 1
            blocks[m.group(1)] = range(i, j)
        i = j
    test_only = set()
    while True:
        inside = {k for f in test_only for k in blocks[f]}
        def only_tests(f):
            uses = 0
            for k, line in enumerate(lines):
                if k in blocks[f] or k in inside or line.strip().startswith("//"):
                    continue
                for m in re.finditer(rf"\b{f}\b", line):
                    before = line[:m.start()]
                    if not (re.search(r"gen:\s*$", before) or " where " in before):
                        return False
                    uses += 1
            return uses > 0 or any(re.search(rf"\b{f}\b", lines[k]) for k in inside)
        grown = {f for f in blocks if f in test_only or only_tests(f)}
        if grown == test_only:
            return {k + 1 for k in inside}
        test_only = grown

def measure(name, src=None):
    src = src or f"{PORTS}/{name}.cp"
    port = counted(src, False)
    orig = counted(f"{PORTS}/{name}_orig.cpp", True)
    gone = set()
    for a, b, _, _ in stripped_ranges(f"{PORTS}/{name}.strip"):
        gone |= set(range(a, b + 1))
    tests = scaffolding(src)
    return len(port), len(orig), len([i for i in orig if i not in gone]), len([i for i in port if i in tests])

def pct(ours, theirs):
    return 100.0 * (theirs - ours) / theirs

def before_after(names):
    """ports that also exist under before/: lines then and now, against the same original"""
    rows = []
    for n in names:
        if os.path.exists(f"{PORTS}/before/{n}.cp"):
            (b, o, s, _), (a, _, _, _) = measure(n, f"{PORTS}/before/{n}.cp"), measure(n)
            rows.append((n, b, a, o, s))
    if not rows:
        return
    rows.append((f"all {len(rows)}", *(sum(r[i] for r in rows) for i in (1, 2, 3, 4))))
    print("\nbefore and after the changes these ports asked for (reductions are against the original):")
    print(f"{'port':<14}{'before':>7}{'after':>7}{'raw before':>12}{'raw after':>11}{'stripped before':>17}{'stripped after':>16}")
    for n, b, a, o, s in rows:
        print(f"{n:<14}{b:>7}{a:>7}{pct(b, o):>11.0f}%{pct(a, o):>10.0f}%{pct(b, s):>16.0f}%{pct(a, s):>15.0f}%")

def main():
    global PORTS
    dirs = [a for a in sys.argv[1:] if not a.startswith("--")]
    if dirs:
        PORTS = os.path.abspath(dirs[0]) if os.path.isdir(dirs[0]) else f"{HERE}/ports/{dirs[0]}"
    names = sorted(os.path.basename(p)[:-3] for p in glob.glob(f"{PORTS}/*.cp"))
    rows = [(n, *measure(n)) for n in names]
    rows.append((f"all {len(rows)}", *(sum(r[i] for r in rows) for i in (1, 2, 3, 4))))
    if "--md" in sys.argv:
        print("| port | port lines | of which test-only | original | raw reduction | original, library-stripped | stripped reduction |")
        print("|---|---:|---:|---:|---:|---:|---:|")
        for n, p, o, s, t in rows:
            print(f"| `{n}` | {p} | {t} | {o} | {pct(p, o):.0f}% | {s} | {pct(p, s):.0f}% |")
    else:
        print(f"{'port':<14}{'port':>6}{'tests':>7}{'orig':>6}{'raw':>8}{'stripped':>10}{'reduction':>11}")
        for n, p, o, s, t in rows:
            print(f"{n:<14}{p:>6}{t:>7}{o:>6}{pct(p, o):>7.0f}%{s:>10}{pct(p, s):>10.0f}%")
    n, p, o, s, t = rows[-1]
    print(f"\nwithout the ports' test-only lines: raw {pct(p - t, o):.0f}%, library-stripped {pct(p - t, s):.0f}%")
    worse = [r[0] for r in rows if r[1] >= r[3]]
    if worse:
        print("no library-stripped win (friction):", ", ".join(worse))
    before_after(names)

if __name__ == "__main__":
    main()
