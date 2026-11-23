#!/usr/bin/env python3
"""Checker for CF 2211 D: any 0 <= a_i < 2^29 whose AND-array matches b mod
1e9+7. Per bit with c set elements, f(a)_k gains 2^bit * C(c, k)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]

MOD = 10**9 + 7


def main():
    inp, out, _ = load()
    t, pi, po = int(inp[0]), 1, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        b = [int(x) for x in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        a = [int(x) for x in out[po:po + n]]
        po += n
        if len(a) != n or any(not 0 <= x < 1 << 29 for x in a):
            fail(f"case {case}: expected {n} values in [0, 2^29)")
        fact = [1] * (n + 1)
        for i in range(1, n + 1):
            fact[i] = fact[i - 1] * i % MOD
        inv = [pow(f, MOD - 2, MOD) for f in fact]
        f = [0] * (n + 1)
        for bit in range(29):
            c = sum(x >> bit & 1 for x in a)
            for k in range(1, c + 1):
                f[k] = (f[k] + (fact[c] * inv[k] % MOD * inv[c - k] << bit)) % MOD
        for k in range(1, n + 1):
            if f[k] != b[k - 1]:
                fail(f"case {case}: f(a)_{k} = {f[k]}, b_{k} = {b[k - 1]}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
