import sys, random
# seed size: a tree of n <= size + 1 nodes (p_i < i: random, chain, star or broom), q <= 4 * size queries
# of a node and a depth 1..=n-1, and M in {1, 5, 50, 500}; --max: n = 2e5 random, q = 1e6, M = 1.
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 200000 if mx else r.randint(2, S + 1)
q = 1000000 if mx else r.randint(1, 4 * S)
M = 1 if mx else r.choice([1, 5, 50, 500])
shape = "random" if mx else r.choice(["random", "random", "chain", "star", "broom"])
if shape == "chain":
    p = [i - 1 for i in range(2, n + 1)]
elif shape == "star":
    p = [1] * (n - 1)
elif shape == "broom":
    h = r.randint(1, n - 1)
    p = [i - 1 if i <= h else r.randint(1, h) for i in range(2, n + 1)]
else:
    p = [r.randint(max(1, i - r.choice([1, 3, i])), i - 1) for i in range(2, n + 1)]
out = [f"{1} {n} {q} {M}", " ".join(map(str, p))]
out += [f"{r.randint(1, n)} {r.randint(1, n - 1)}" for _ in range(q)]
print("\n".join(out))
