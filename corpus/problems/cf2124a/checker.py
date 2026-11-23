#!/usr/bin/env python3
"""Checker for CF 2124 A: YES exactly when the reference says YES (any case),
then any non-empty subsequence of a that is a derangement of its sorted order."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def main():
    inp, out, ref = load()
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        a = [int(x) for x in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        if po >= len(out):
            fail(f"case {case}: output ends early")
        want = ref[pr].upper() == "YES"
        pr += 1 + (1 + int(ref[pr + 1]) if want else 0)
        got = out[po].upper()
        po += 1
        if got not in ("YES", "NO"):
            fail(f"case {case}: expected YES or NO, got {out[po - 1]}")
        if (got == "YES") != want:
            fail(f"case {case}: answered {got}, the answer is {'YES' if want else 'NO'}")
        if not want:
            continue
        k = int(out[po])
        d = [int(x) for x in out[po + 1:po + 1 + k]]
        po += 1 + k
        if not 1 <= k <= n or len(d) != k:
            fail(f"case {case}: k = {k} is not in 1..{n}")
        it = iter(a)
        if not all(any(x == y for y in it) for x in d):
            fail(f"case {case}: d is not a subsequence of a")
        if any(x == y for x, y in zip(d, sorted(d))):
            fail(f"case {case}: d is not a derangement")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
