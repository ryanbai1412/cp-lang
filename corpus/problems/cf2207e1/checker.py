#!/usr/bin/env python3
"""Checker for CF 2207 E1: YES exactly when the reference says YES, then
0 <= b_i <= 1e9 with the (n - i + 1)-mex of b_1..b_i equal to a_i. Values above
2n never change a k-mex with k <= n, so a Fenwick tree of missing values in
0..2n finds each k-th missing value."""
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
        want = ref[pr].upper() == "YES"
        pr += 1 + (n if want else 0)
        got = out[po].upper()
        po += 1
        if got not in ("YES", "NO") or (got == "YES") != want:
            fail(f"case {case}: answered {out[po - 1]}, the answer is {'YES' if want else 'NO'}")
        if not want:
            continue
        b = [int(x) for x in out[po:po + n]]
        po += n
        if len(b) != n or any(not 0 <= x <= 10**9 for x in b):
            fail(f"case {case}: expected {n} values in [0, 1e9]")
        size = 2 * n + 2
        tree = [0] * (size + 1)
        for i in range(1, size + 1):
            tree[i] += 1
            j = i + (i & -i)
            if j <= size:
                tree[j] += tree[i]
        present = set()
        log = size.bit_length()
        for i, x in enumerate(b, 1):
            if x < size and x not in present:
                present.add(x)
                j = x + 1
                while j <= size:
                    tree[j] -= 1
                    j += j & -j
            k, pos = n - i + 1, 0
            for step in range(log, -1, -1):
                nxt = pos + (1 << step)
                if nxt <= size and tree[nxt] < k:
                    pos = nxt
                    k -= tree[nxt]
            if pos != a[i - 1]:
                fail(f"case {case}: the {n - i + 1}-mex of b_1..b_{i} is {pos}, not {a[i - 1]}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
