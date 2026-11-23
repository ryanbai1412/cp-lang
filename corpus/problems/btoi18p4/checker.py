#!/usr/bin/env python3
"""Checker for BOI 2018 "Alternating Current": "impossible" exactly when the
reference says so, else m digits where the wires of each direction cover all n
segments of the ring."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp, out, ref = [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]
    n, m = int(inp[0]), int(inp[1])
    if len(out) != 1:
        fail(f"expected one token, got {len(out)}")
    if ref[0] == "impossible" or out[0] == "impossible":
        if out[0] != ref[0]:
            fail(f"printed {out[0][:20]!r}, reference {ref[0][:20]!r}")
        return
    s = out[0]
    if len(s) != m or set(s) - {"0", "1"}:
        fail(f"need {m} digits 0/1")
    diff = [[0] * (n + 1) for _ in range(2)]
    for i in range(m):
        a, b = int(inp[2 + 2 * i]) - 1, int(inp[3 + 2 * i]) - 1
        d = diff[int(s[i])]
        if a <= b:
            d[a] += 1
            d[b + 1] -= 1
        else:
            d[a] += 1
            d[n] -= 1
            d[0] += 1
            d[b + 1] -= 1
    for side in range(2):
        cur = 0
        for seg in range(n):
            cur += diff[side][seg]
            if cur == 0:
                fail(f"direction {side} misses segment {seg + 1}")


if __name__ == "__main__":
    main()
