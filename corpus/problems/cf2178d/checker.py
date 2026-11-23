#!/usr/bin/env python3
"""Checker for CF 2178 D: -1 exactly when the reference says -1, else a legal
attack sequence (both alive, attacker fresh) after which exactly m elves live
and no living fresh elf has a living target."""
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
        n, m = int(inp[pi]), int(inp[pi + 1])
        a = [0] + [int(x) for x in inp[pi + 2:pi + 2 + n]]
        pi += 2 + n
        none_ref = ref[pr] == "-1"
        pr += 1 if none_ref else 1 + 2 * int(ref[pr])
        if po >= len(out):
            fail(f"case {case}: output ends early")
        if out[po] == "-1":
            if not none_ref:
                fail(f"case {case}: printed -1 but a sequence exists")
            po += 1
            continue
        if none_ref:
            fail(f"case {case}: no sequence exists, expected -1")
        k = int(out[po])
        po += 1
        if not 0 <= k <= n:
            fail(f"case {case}: k = {k} out of range")
        h, used = a[:], [False] * (n + 1)
        for i in range(1, k + 1):
            x, y = int(out[po]), int(out[po + 1])
            po += 2
            if not (1 <= x <= n and 1 <= y <= n and x != y):
                fail(f"case {case}, attack {i}: bad pair {x} {y}")
            if h[x] <= 0 or h[y] <= 0 or used[x]:
                fail(f"case {case}, attack {i}: {x} -> {y} is not allowed")
            used[x] = True
            h[y] -= a[x]
            h[x] -= a[y]
        alive = [v for v in range(1, n + 1) if h[v] > 0]
        if len(alive) != m:
            fail(f"case {case}: {len(alive)} alive, expected {m}")
        if len(alive) >= 2 and any(not used[v] for v in alive):
            fail(f"case {case}: the process has not terminated")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
