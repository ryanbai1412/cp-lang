import sys, random
# gen.py SEED SIZE: n, m <= SIZE (capped at 1e5; each is SIZE 30% of the time), distinct i, j, k.
# Most cases plant a hidden array (values 1..1e9, often few distinct so pins repeat) and take each
# query's true median, so the answer is YES; some then corrupt one median, and some draw queries
# at random (usually NO).
rng = random.Random(int(sys.argv[1])); S = max(3, min(int(sys.argv[2]), 100000))
n = S if rng.random() < 0.3 else rng.randint(3, S)
m = S if rng.random() < 0.3 else rng.randint(1, S)
vmax = rng.choice([3, 10, n, 10**9])
vals = sorted(rng.sample(range(1, 10**9 + 1), min(vmax, 50)))
a = [rng.choice(vals) if vmax < 10**9 else rng.randint(1, 10**9) for _ in range(n)]
kind = rng.randrange(4)
qs = []
for _ in range(m):
    i, j, k = rng.sample(range(n), 3)
    x = sorted([a[i], a[j], a[k]])[1] if kind < 3 else rng.choice(vals)
    qs.append([i + 1, j + 1, k + 1, x])
if kind == 2:
    q = rng.choice(qs); q[3] = max(1, min(10**9, q[3] + rng.choice([-1, 1])))
print(n, m)
print('\n'.join(' '.join(map(str, q)) for q in qs))
