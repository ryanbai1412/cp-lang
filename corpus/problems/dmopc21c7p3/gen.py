import sys, random
# statement: N <= 1e6 figures k_i in 0..=1, M <= 1e6 distinct links a -> b, a != b.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
n = 10 ** 6 if big else r.randint(2, max(2, min(S, 8)))
acyclic = big or r.random() < 0.6
m = 10 ** 6 if big else r.randint(0, min(n * (n - 1) // (2 if acyclic else 1), 2 * n))
rank = list(range(1, n + 1)); r.shuffle(rank)
links = set()
while len(links) < m:
    a, b = r.sample(range(1, n + 1), 2) if not big else (r.randint(1, n), r.randint(1, n))
    if a == b:
        continue
    if acyclic and rank[a - 1] > rank[b - 1]:
        a, b = b, a
    links.add((a, b))
print(n, len(links))
print(*(int(r.random() < 0.5) for _ in range(n)))
print("\n".join(f"{a} {b}" for a, b in links))
