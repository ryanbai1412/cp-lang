#!/usr/bin/env python3
"""Line counts for scout batch 35, measured two ways.

raw       port lines against the accepted C++ submission
stripped  port lines against the submission minus what a perfect C++ contest
          library would have supplied (the ranges in <code>.strip)

Blank and comment-only lines never count on either side. Both the v2 port
and its runnable today/ form are counted.

    python3 proto/ports/scout_batch_35/loc.py
"""
import glob, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))


def counted(path):
    out, inside = [], False
    for i, line in enumerate(open(path, errors="replace").read().split("\n"), 1):
        s = line.strip()
        if inside:
            inside = "*/" not in s
        elif s.startswith("/*"):
            inside = "*/" not in s
        elif s and not s.startswith("//"):
            out.append(i)
    return out


def main():
    tot = [0, 0, 0, 0]
    print(f"{'problem':<12}{'v2':>5}{'today':>7}{'C++':>6}{'C++ stripped':>14}{'raw cut':>9}{'stripped cut':>14}")
    for strip in sorted(glob.glob(os.path.join(HERE, "*.strip"))):
        code = os.path.basename(strip)[:-6]
        text = open(strip).read()
        sub = re.search(r"submissions/(\d+)\.cpp", text).group(1)
        gone = set()
        for row in text.split("\n"):
            if row.strip() and not row.startswith("#"):
                a, b = map(int, row.split()[0].split("-"))
                gone |= set(range(a, b + 1))
        cpp = counted(os.path.join(ROOT, "corpus", "submissions", sub + ".cpp"))
        row = [len(counted(os.path.join(HERE, code + ".cp"))), len(counted(os.path.join(HERE, "today", code + ".cp"))),
               len(cpp), len([i for i in cpp if i not in gone])]
        tot = [a + b for a, b in zip(tot, row)]
        show(code, row)
    show("total", tot)


def show(name, r):
    print(f"{name:<12}{r[0]:>5}{r[1]:>7}{r[2]:>6}{r[3]:>14}{100 - 100 * r[0] / r[2]:>8.0f}%{100 - 100 * r[0] / r[3]:>13.0f}%")


if __name__ == "__main__":
    main()
