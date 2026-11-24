"""yac1p2 inputs: `gen.py seed size` gives a random tree on 3..max(3, size) nodes (random
parents, or a path for seed % 3 == 0) with random B/W colours, mostly one colour for
seed % 4 == 1. `gen.py --max` gives n = 2e5."""
import random, sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
seed, size = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
n = size if size >= 200000 else r.randint(3, max(3, size))
p = 0.9 if seed % 4 == 1 else 0.5
print(n)
print("".join("B" if r.random() < p else "W" for _ in range(n)))
perm = list(range(1, n + 1)); r.shuffle(perm)
for i in range(1, n):
    j = i - 1 if seed % 3 == 0 else r.randrange(i)
    print(perm[i], perm[j])
