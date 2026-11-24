#!/usr/bin/env python3
"""gen.py seed size | --max: NOIP 2020 Drainage. n <= 1e5 junctions, junctions 1..m (m <= 10)
take one ton each, out-degree 0 to 5, at most 10 inner junctions on any path. --max: n = 1e5,
m = 10, ten full inner layers between the sources and the sinks with out-degrees 3, 4 and 5
(denominators up to 60^10) and links that skip layers."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100_000 if MAX else max(2, min(100_000, int(sys.argv[2])))
m = 10 if MAX else r.randint(1, min(10, n - 1))
inner = 10 if MAX else r.randint(0, 10)
rest = list(range(m + 1, n + 1)); r.shuffle(rest)
layers = [list(range(1, m + 1))]
size = max(1, (n - m) // (inner + 1))
for i in range(inner + 1):
    chunk = rest[i * size:] if i == inner else rest[i * size:(i + 1) * size]
    if chunk: layers.append(chunk)
out = {v: [] for v in range(1, n + 1)}
for i, layer in enumerate(layers[:-1]):
    for v in layer:
        d = r.choice([3, 4, 5]) if MAX else r.randint(1, 5)
        for _ in range(d):
            j = i + 1 if r.random() < 0.9 else r.randint(i + 1, len(layers) - 1)
            out[v].append(r.choice(layers[j]))
        out[v] = list(dict.fromkeys(out[v]))
print(n, m)
for v in range(1, n + 1):
    print(len(out[v]), *out[v])
