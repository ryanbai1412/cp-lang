#!/usr/bin/env python3
"""Checker for CF 2124 I: YES exactly when the reference says YES, then a
permutation with f(a_1..a_i) = x_i. The lexicographically largest b starts at
a_1; from the element just taken, the next part may start anywhere up to and
including the first smaller element, so the greedy takes the largest of those.
f is the number of steps; this is O(n^2) per prefix, enough for the corpus's
generated sizes."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def f(a):
    i, k = 0, 1
    while i + 1 < len(a):
        best, j = i + 1, i + 1
        while j < len(a):
            if a[j] > a[best]:
                best = j
            if a[j] < a[i]:
                break
            j += 1
        i, k = best, k + 1
    return k


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp, out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[1:])
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        x = [int(v) for v in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        want = ref[pr].upper() == "YES"
        pr += 1 + (n if want else 0)
        got = out[po].upper()
        po += 1
        if got not in ("YES", "NO") or (got == "YES") != want:
            fail(f"case {case}: answered {out[po - 1]}, the answer is {'YES' if want else 'NO'}")
        if not want:
            continue
        a = [int(v) for v in out[po:po + n]]
        po += n
        if len(a) != n or sorted(a) != list(range(1, n + 1)):
            fail(f"case {case}: not a permutation of 1..{n}")
        for i in range(1, n + 1):
            if f(a[:i]) != x[i - 1]:
                fail(f"case {case}: f of the first {i} is {f(a[:i])}, not {x[i - 1]}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
