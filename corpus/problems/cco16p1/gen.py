"""gen.py SEED SIZE | --max: CCO 2016 P1 (Field Trip). N <= 1e6 students, M <= 1e6 friendships,
each student in at most two, so the friendships form paths and cycles; buses of K. Small cases
mix short paths, cycles and loners under shuffled labels. --max is N = M = 1e6: cycles only,
of random lengths, K = 7."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1_000_000 if MAX else r.randint(1, 2 + 3 * S)
label = list(range(1, n + 1))
r.shuffle(label)
edges, i = [], 0
while i < n:
    left = n - i
    s = min(left, r.randint(1, 30) if MAX else r.randint(1, 2 + S))
    if MAX and left - s < 3:
        s = left
    cycle = s >= 3 and (MAX or r.random() < 0.4)
    part = label[i:i + s]
    edges += [(part[j], part[j + 1]) for j in range(s - 1)]
    if cycle:
        edges.append((part[-1], part[0]))
    i += s
r.shuffle(edges)
edges = [(a, b) if r.random() < 0.5 else (b, a) for a, b in edges]
k = 7 if MAX else r.randint(1, min(n, 1 + S))
sys.stdout.write(f"{n} {len(edges)} {k}\n" + "".join(f"{a} {b}\n" for a, b in edges))
