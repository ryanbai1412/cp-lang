#!/usr/bin/env python3
"""Dual LOC for scout batch 49.

A line counts when it is neither blank nor a pure `//` comment.
  raw       the whole original C++ file
  stripped  the original as if C++ had a perfect contest library: without
            its template (everything up to and including `#define sz(`, and
            the stream setup lines in main), without debug printers, and
            without the stock data structures listed in LIBRARY below
            (problem-specific node types such as `struct Node` stay)
  v2        <problem>.cp, the v2-spec port
  today     today/<problem>.cp, the form proto/cpc builds now
"""
import os, re

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {"ioi00p2": "4682555", "ioi01p3": "4806167", "ioi99p6": "4899527", "ioi14p4": "4684567",
        "ioi15p5": "5723358", "ioi17p2": "5724153", "ioi21p1": "4698803", "ioi22p4": "4783516",
        "ioi12p2": "4689474", "ioi18p2": "4731161"}
# top-level blocks a library would provide: first line of the block -> what it is
LIBRARY = {
    "ioi99p6": ["template <bool mn>"],                 # sparse table
    "ioi14p4": ["long bpow("],                         # modular power
    "ioi15p5": ["template <typename T>", "template <typename A, typename B>"],   # debug printers
    "ioi17p2": ["struct PSA {"],                       # prefix sums
    "ioi21p1": ["struct DS {"],                        # segment tree with descent
    "ioi22p4": ["struct DS {", "pair<long, long> operator+("],   # lazy segment tree, pair sum
    "ioi12p2": ["struct DSU {"],                       # union-find
    "ioi18p2": ["struct DS {"],                        # lazy segment tree
}
IO_SETUP = re.compile(r"sync_with_stdio|cin\.tie|cin\.exceptions|freopen")


def code(lines):
    return [l for l in lines if l.strip() and not l.strip().startswith("//")]


def read(path):
    return open(path, encoding="utf-8", errors="replace").read().splitlines()


def without_blocks(lines, starts):
    out, skipping = [], False
    for l in lines:
        if not skipping and any(l.startswith(s) for s in starts): skipping = True
        if not skipping: out.append(l)
        elif l.startswith("}"): skipping = False
    return out


tot = [0, 0, 0, 0]
print(f"{'problem':10} {'raw':>5} {'stripped':>8} {'v2':>4} {'today':>5}")
for name, sub in ORIG.items():
    cpp = read(os.path.join(ROOT, "corpus", "submissions", sub + ".cpp"))
    cut = max((i + 1 for i, l in enumerate(cpp) if l.startswith("#define sz(")), default=0)
    body = without_blocks(cpp[cut:], LIBRARY.get(name, []))
    row = [
        len(code(cpp)),
        len([l for l in code(body) if not IO_SETUP.search(l)]),
        len(code(read(os.path.join(HERE, name + ".cp")))),
        len(code(read(os.path.join(HERE, "today", name + ".cp")))),
    ]
    tot = [a + b for a, b in zip(tot, row)]
    print(f"{name:10} {row[0]:5} {row[1]:8} {row[2]:4} {row[3]:5}")
print(f"{'total':10} {tot[0]:5} {tot[1]:8} {tot[2]:4} {tot[3]:5}")
print(f"v2 vs raw: -{100 - 100 * tot[2] / tot[0]:.0f}%   v2 vs stripped: -{100 - 100 * tot[2] / tot[1]:.0f}%")
print(f"today vs raw: -{100 - 100 * tot[3] / tot[0]:.0f}%   today vs stripped: -{100 - 100 * tot[3] / tot[1]:.0f}%")
