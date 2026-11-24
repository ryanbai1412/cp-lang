#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SHOI 2015 Brain Hole Therapy Device. n, m <= 2e5 cells and
operations: `0 l r`, `1 l0 r0 l1 r1`, `2 l r`, every range inside 1..=n. --max: n = m = 2e5."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 200000 if MAX else min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 200000)
n, m = (size, size) if MAX else (r.randint(1, size), r.randint(1, size))
def seg():
    a, b = r.randint(1, n), r.randint(1, n)
    return f"{min(a, b)} {max(a, b)}"
out = [f"{n} {m}"]
for _ in range(m):
    k = r.randint(0, 2)
    out.append(f"1 {seg()} {seg()}" if k == 1 else f"{k} {seg()}")
print("\n".join(out))
