#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SAC '22 CC2 P3. A perfect binary decision tree of depth N <= 19
on nodes 1..2^(N+1)-1 (root 1, the rest shuffled), its 2^N - 1 forks `u a b w` in random
order, |w| <= 1e9. --max: N = 19."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 1 << 20 if MAX else (int(sys.argv[2]) if len(sys.argv) > 2 else 8)
top = max(1, min(19, size.bit_length() - 1))
k = 19 if MAX else r.randint(1, top)
nodes = 2 ** (k + 1) - 1
label = [1] + r.sample(range(2, nodes + 1), nodes - 1)
span = r.choice([10, 10**4, 10**9])
forks = [(label[i], label[2 * i + 1], label[2 * i + 2], r.randint(-span, span)) for i in range(2**k - 1)]
r.shuffle(forks)
print(k, r.randint(0, 10**9))
print("\n".join(f"{u} {a} {b} {w}" for u, a, b, w in forks))
