#!/usr/bin/env python3
"""Dual LOC for scout batch 45.

A line counts when it is neither blank nor a pure `//` comment.
  raw       the whole original C++ file
  stripped  the original without its contest template: everything up to and
            including the `#define sz(x)` line (includes, debug macros,
            aliases) and the stream setup lines in main
  v2        <problem>.cp, the v2-spec port
  today     today/<problem>.cp, the form proto/cpc builds now
"""
import os, re

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {"shoi15p2": "5450923", "shoi16p1": "5445691", "shoi16p2": "5445692", "shoi16p5": "5445695",
        "ucrpc21h": "3950286", "iati21a12": "5071843", "iati21a23": "5072008", "neerc10k": "5325567",
        "rpeng2": "4810287", "noimps22p5": "5448138"}
IO_SETUP = re.compile(r"sync_with_stdio|cin\.tie|cin\.exceptions|freopen")


def code(lines):
    return [l for l in lines if l.strip() and not l.strip().startswith("//")]


def read(path):
    return open(path, encoding="utf-8", errors="replace").read().splitlines()


tot = [0, 0, 0, 0]
print(f"{'problem':10} {'raw':>5} {'stripped':>8} {'v2':>4} {'today':>5}")
for name, sub in ORIG.items():
    cpp = read(os.path.join(ROOT, "corpus", "submissions", sub + ".cpp"))
    cut = max((i + 1 for i, l in enumerate(cpp) if l.startswith("#define sz(")), default=0)
    row = [
        len(code(cpp)),
        len([l for l in code(cpp[cut:]) if not IO_SETUP.search(l)]),
        len(code(read(os.path.join(HERE, name + ".cp")))),
        len(code(read(os.path.join(HERE, "today", name + ".cp")))),
    ]
    tot = [a + b for a, b in zip(tot, row)]
    print(f"{name:10} {row[0]:5} {row[1]:8} {row[2]:4} {row[3]:5}")
print(f"{'total':10} {tot[0]:5} {tot[1]:8} {tot[2]:4} {tot[3]:5}")
print(f"v2 vs raw: -{100 - 100 * tot[2] / tot[0]:.0f}%   v2 vs stripped: -{100 - 100 * tot[2] / tot[1]:.0f}%")
print(f"today vs raw: -{100 - 100 * tot[3] / tot[0]:.0f}%   today vs stripped: -{100 - 100 * tot[3] / tot[1]:.0f}%")
