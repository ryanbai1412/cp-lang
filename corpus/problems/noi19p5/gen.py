#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2019 Landlords. "n m type", m cuts 0 <= A_i <= n, then Q and Q
positions 1..n. Small: n <= size + 3, m, Q <= size + 1. --max: n = 1e7, m = Q = 5e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10**7 if MAX else r.randint(3, S + 3)
m = 5 * 10**5 if MAX else r.randint(1, S + 1)
q = 5 * 10**5 if MAX else r.randint(1, S + 1)
out = [f"{n} {m} {r.randint(1, 2)}"]
out += [str(r.choice([0, n, r.randint(0, n)]) if r.random() < 0.1 else r.randint(1, n - 1)) for _ in range(m)]
out.append(str(q))
out += [str(r.randint(1, n)) for _ in range(q)]
print("\n".join(out))
