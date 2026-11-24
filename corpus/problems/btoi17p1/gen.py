"""btoi17p1 (Political Development) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, K, then per politician the count and list of those they disagree with (0-based,
symmetric). Each politician, in a hidden order, disagrees with at most K - 1 earlier ones, so
every group has a member with fewer than K disagreements inside it. --max: N = 5e4, K = 10,
each linked to 0..9 of the 12 before it (all 9 would take the accepted C++ about 9 s).
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, k, window = 5 * 10**4, 10, 12
else:
    n = rng.randint(1, max(1, min(5 * 10**4, int(sys.argv[2]))))
    k = rng.randint(1, 10)
    window = rng.randint(k, 2 * k + 2)
label = list(range(n))
rng.shuffle(label)
adj = [set() for _ in range(n)]
for i in range(1, n):
    earlier = list(range(max(0, i - window), i))
    want = rng.randint(0, k - 1)
    for j in rng.sample(earlier, min(want, len(earlier))):
        adj[label[i]].add(label[j])
        adj[label[j]].add(label[i])
print(n, k)
for a in adj:
    a = list(a)
    rng.shuffle(a)
    print(len(a), *a)
