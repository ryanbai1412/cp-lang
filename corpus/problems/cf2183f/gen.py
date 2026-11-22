import sys, random

seed, S = (7, 150) if "--max" in sys.argv[1:] else (int(sys.argv[1]), int(sys.argv[2]))
r = random.Random(seed)
full = S >= 8 and seed % 64 == 7
budget = 5000 if full else min(5000, 1 + S * S)
tests = []
while budget > 0 and len(tests) < (1 if full else 6):
    n = budget if full else r.randint(1, budget)
    tests.append(n)
    budget -= n
print(len(tests))
for n in tests:
    letters = "abcdefghijklmnopqrstuvwxyz"[: r.choice([1, 2, 3, 26])]
    shape = r.choice(["random", "path", "star", "broom"])
    print(n)
    print("".join(r.choice(letters) for _ in range(n)))
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    perm.remove(1)
    perm = [1] + perm
    for i in range(1, n):
        if shape == "path":
            p = i - 1
        elif shape == "star":
            p = 0
        elif shape == "broom":
            p = i - 1 if i < n // 2 else r.randrange(i)
        else:
            p = r.randrange(max(0, i - 3), i) if r.random() < 0.5 else r.randrange(i)
        u, v = perm[p], perm[i]
        print(*((u, v) if r.random() < 0.5 else (v, u)))
