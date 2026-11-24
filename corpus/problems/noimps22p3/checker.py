#!/usr/bin/env python3
"""Checker for NOI 2022 multi-provincial selection P3: per case the maximum count must equal the
answer's, and the order must be a permutation of 1..M under which exactly that many louxia /
loushang messages hold."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = open(sys.argv[1]).read().split()
    out = open(sys.argv[2]).read().split()
    ans = open(sys.argv[3]).read().split()
    pi = po = pa = 0
    t = int(inp[pi])
    pi += 1
    for c in range(1, t + 1):
        n, m = int(inp[pi]), int(inp[pi + 1])
        pi += 2
        names = set(inp[pi : pi + n])
        pi += n
        msgs = [inp[pi + 3 * i : pi + 3 * i + 3] for i in range(m)]
        pi += 3 * m
        want = int(ans[pa])
        pa += 1 + m
        try:
            got = int(out[po])
            perm = [int(x) - 1 for x in out[po + 1 : po + 1 + m]]
        except (ValueError, IndexError):
            fail(f"case {c}: malformed output")
        po += 1 + m
        if len(perm) != m or sorted(perm) != list(range(m)):
            fail(f"case {c}: not a permutation of 1..{m}")
        if got != want:
            fail(f"case {c}: count {got}, expected {want}")
        real = 0
        for i, z in enumerate(perm):
            a, b, kind = msgs[z]
            if b not in names:
                continue
            if kind == "louxia" and i > 0 and msgs[perm[i - 1]][0] == b:
                real += 1
            if kind == "loushang" and i + 1 < m and msgs[perm[i + 1]][0] == b:
                real += 1
        if real != got:
            fail(f"case {c}: the order earns {real}, not {got}")
    if po != len(out):
        fail("trailing output")


if __name__ == "__main__":
    main()
