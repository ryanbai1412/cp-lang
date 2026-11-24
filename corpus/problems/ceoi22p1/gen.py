#!/usr/bin/env python3
"""gen.py seed size | --max: CEOI 2022 Abracadabra. Even N <= 2e5 cards (a permutation),
Q <= 1e6 queries t <= 1e9, 1 <= i <= N. --max: N = 2e5, Q = 1e6, the deck shuffled with long
descending runs so many shuffles change it, times spread from 0 to 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 200_000 if MAX else 2 * r.randint(1, max(1, int(sys.argv[2]) // 2))
q = 1_000_000 if MAX else r.randint(1, max(1, int(sys.argv[2])))
deck = list(range(1, n + 1))
r.shuffle(deck)
T = 10 ** 9 if MAX or r.random() < 0.3 else n
print(n, q)
print(*deck)
print("\n".join(f"{r.choice([0, 1, 2, r.randint(0, T)])} {r.randint(1, n)}" for _ in range(q)))
