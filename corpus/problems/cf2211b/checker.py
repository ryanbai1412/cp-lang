#!/usr/bin/env python3
"""Checker for CF 2211 B: the reference's minimum f(a) mod 676767677, and an
arrangement of x ones and y minus ones whose own f equals it. f counts
partitions into m equal-sum parts: for m parts of sum s = S / m, the cut after
position i needs prefix j * s, and ways flow from prefix (j - 1) * s to it."""
import sys

MOD = 676767677


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def f(a):
    n, pre = len(a), [0]
    for v in a:
        pre.append(pre[-1] + v)
    total = pre[-1]
    if total == 0:
        return pow(2, sum(1 for p in pre[1:n] if p == 0), MOD)
    count = 0
    for m in range(1, abs(total) + 1):
        if total % m:
            continue
        s, ways = total // m, {0: 1}
        for p in pre[1:n]:
            if p % s == 0 and 0 < p // s < m:
                ways[p] = (ways.get(p, 0) + ways.get(p - s, 0)) % MOD
        count += ways.get(total - s, 0) if m > 1 else 1
    return count % MOD


def main():
    inp, out, ref = load()
    t, po, pr = int(inp[0]), 0, 0
    for case in range(1, t + 1):
        x, y = int(inp[2 * case - 1]), int(inp[2 * case])
        n = x + y
        val, want = int(out[po]), int(ref[pr])
        a = [int(v) for v in out[po + 1:po + 1 + n]]
        po, pr = po + 1 + n, pr + 1 + n
        if val != want:
            fail(f"case {case}: printed {val}, the minimum is {want}")
        if len(a) != n or a.count(1) != x or a.count(-1) != y:
            fail(f"case {case}: expected {x} ones and {y} minus ones")
        if f(a) != val:
            fail(f"case {case}: the array has f = {f(a)}, not {val}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
