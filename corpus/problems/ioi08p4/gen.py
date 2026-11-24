#!/usr/bin/env python3
"""gen.py seed size | --max: IOI 2008 Teleporters. n, m <= 1e6, n teleporters "w e" with
1 <= w < e <= 2e6 and all 2n endpoints distinct. --max: n = m = 1e6, every point 1..=2e6 used,
paired at random (many cycles of every length) so the m new teleporters splice them all."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 1_000_000 if MAX else max(1, min(1_000_000, int(sys.argv[2])))
m = 1_000_000 if MAX else r.randint(1, max(1, n))
pts = list(range(1, 2 * n + 1)) if MAX else r.sample(range(1, 2_000_001), 2 * n)
r.shuffle(pts)
lines = [f"{min(pts[2 * i], pts[2 * i + 1])} {max(pts[2 * i], pts[2 * i + 1])}" for i in range(n)]
sys.stdout.write(f"{n}\n{m}\n" + "\n".join(lines) + "\n")
