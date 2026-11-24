"""gen.py seed size: DMOPC '20 C7 P4. 1 <= N, Q <= 2e5, a tree; "1 u" toggles, "2 u v" asks.
size >= 2e5 gives N = Q = 2e5: by seed, a random tree, a path, or a star."""
import random, sys
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 2 * 10 ** 5
n = 2 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 10)))
q = 2 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 15)))
shape = seed % 3 if big else rng.randrange(3)
perm = list(range(1, n + 1)); rng.shuffle(perm)
out = [f"{n} {q}"]
for i in range(1, n):
    j = rng.randrange(i) if shape == 0 else (i - 1 if shape == 1 else 0)
    out.append(f"{perm[i]} {perm[j]}")
for _ in range(q):
    if rng.random() < 0.5: out.append(f"1 {rng.randint(1, n)}")
    else: out.append(f"2 {rng.randint(1, n)} {rng.randint(1, n)}")
print("\n".join(out))
