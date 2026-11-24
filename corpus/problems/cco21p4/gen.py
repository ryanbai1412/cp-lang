"""CCO 2021 P4 inputs: `gen.py seed size` (a random graph on few cities, with sinks, cycles
and repeated roads), or `gen.py --max` (n = m = 2e5: a long cycle with chords and tails)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
if MAX:
    n = m = 200000; big = 10**9
else:
    n = r.randint(1, max(1, S)); m = r.randint(1, max(1, 2 * S)); big = r.choice([5, 100, 10**9])
roads = []
for i in range(m):
    if MAX and i < n // 2:
        a, b = i + 1, (i + 1) % (n // 2) + 1
    else:
        a, b = r.randint(1, n), r.randint(1, n)
    roads.append(f"{a} {b} {r.randint(0, big)} {r.randint(0, big)}")
print(n, m)
print("\n".join(roads))
