#!/usr/bin/env python3
"""Checker for SDOI 2017 Text Correction: per case the verdict must match the reference's,
and a YES must give three nonempty ranges that cut T into consecutive pieces whose
concatenation in the printed order is S."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp, out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[1:])
    pos = [0, 0]

    def tok(which, src):
        if pos[which] >= len(src):
            fail("output ends early")
        pos[which] += 1
        return src[pos[which] - 1]

    it = iter(inp)
    cases = int(next(it))
    ri = 0
    for case in range(1, cases + 1):
        n = int(next(it))
        next(it)
        S = [next(it) for _ in range(n)]
        T = [next(it) for _ in range(n)]
        want = ref[ri]
        ri += 7 if want == "YES" else 1
        got = tok(0, out)
        if got not in ("YES", "NO"):
            fail(f"case {case}: expected YES or NO, got {got!r}")
        if got != want:
            fail(f"case {case}: answered {got}, the answer is {want}")
        if got == "NO":
            continue
        try:
            segs = [(int(tok(0, out)), int(tok(0, out))) for _ in range(3)]
        except ValueError:
            fail(f"case {case}: ranges must be integers")
        if sorted(segs) != sorted(segs) or any(not 1 <= l <= r <= n for l, r in segs):
            fail(f"case {case}: a range is empty or outside 1..{n}")
        cuts = sorted(segs)
        if cuts[0][0] != 1 or cuts[1][0] != cuts[0][1] + 1 or cuts[2][0] != cuts[1][1] + 1 or cuts[2][1] != n:
            fail(f"case {case}: the ranges do not cut T into three consecutive pieces")
        if [x for l, r in segs for x in T[l - 1:r]] != S:
            fail(f"case {case}: the pieces in this order do not spell S")
    if pos[0] != len(out):
        fail("extra output")


if __name__ == "__main__":
    main()
