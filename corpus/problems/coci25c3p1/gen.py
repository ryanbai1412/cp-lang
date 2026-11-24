#!/usr/bin/env python3
"""COCI 2025/26 R3 Kist: n odd, 1 <= n, k <= 50, a word of 1..=50 capital letters."""
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 49 (the largest odd n), k = 50, 50 paint letters: every paint covers the whole grid
    g = random.Random(1)
    print(49, 50)
    print("".join(g.choice("ABCEFGHIJKMNOPQSTVWXYZ") for _ in range(50)))
    sys.exit()

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 1000:
    n, k, ln = 49, r.choice([1, 25, 50]), 50
    alpha = r.choice(["ABCXYZ", "LRUDAB", "LLLLLLLLLLLLLLLLLLLLLLLLLRRUUDDZ"])
else:
    n = r.choice([1, 3, 5, 7, 9, 11])
    k, ln = r.randint(1, 8), r.randint(1, 20)
    alpha = "LRUDLRUDABCQ"
print(n, k)
print("".join(r.choice(alpha) for _ in range(ln)))
