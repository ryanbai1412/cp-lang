#!/usr/bin/env python3
"""Deterministic input generator for the slices_views zero-cost pair."""
import random

rng = random.Random(202604)
N = 40_000
Q = 2_000
print(N, Q)
print(" ".join(str(rng.randint(1, 10**6)) for _ in range(N)))
for _ in range(Q):
    l = rng.randint(0, N - 1)
    r = rng.randint(l + 1, min(N, l + 20_000))
    print(l, r)
