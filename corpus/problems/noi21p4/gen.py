#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2021 Quantum Communication. n <= 4e5 dictionary words from the
statement's xorshift seeded with a1 a2, m <= 1.2e5 queries "HEX k" with k <= 15, each query's bits
flipped after an answer of 1. --max: n = 4e5, m = 1.2e5, k = 15 mostly: a third of the queries are
dictionary words with <= k flipped bits (answer 1, so the next is flipped), a third near misses with
16..=24 flipped bits (blocks still shared, so candidates get scanned) and the rest random."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 400_000 if MAX else r.randint(1, max(1, int(sys.argv[2])))
m = 120_000 if MAX else r.randint(1, max(1, int(sys.argv[2])))
M64, ALL = (1 << 64) - 1, (1 << 256) - 1
a1, a2 = r.getrandbits(64) | 1, r.getrandbits(64) | 1
k1, k2 = a1, a2
def next_bit():
    global k1, k2
    k3, k4 = k1, k2
    k1 = k4
    k3 ^= (k3 << 23) & M64
    k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26)
    return ((k2 + k4) & M64) >> 32 & 1
known = []
for _ in range(min(n, 1000)):
    w = 0
    for _ in range(256): w = w << 1 | next_bit()
    known.append(w)
def flips(w, c):
    for b in r.sample(range(256), c): w ^= 1 << b
    return w
out = [f"{n} {m} {a1} {a2}"]
flip = False
for _ in range(m):
    k = 15 if MAX and r.random() < 0.8 else r.randint(0, 15)
    kind = r.randrange(3)
    if kind == 0: w, close = flips(r.choice(known), r.randint(0, k)), True
    elif kind == 1: w, close = flips(r.choice(known), r.randint(k + 1, k + 9)), False
    else: w, close = r.getrandbits(256), False
    out.append(f"{format(w ^ ALL if flip else w, '064X')} {k}")
    flip = close
sys.stdout.write("\n".join(out) + "\n")
