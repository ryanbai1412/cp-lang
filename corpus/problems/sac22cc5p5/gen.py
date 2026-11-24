import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "5000"]
# `seed size`: an n x m grid, n, m <= size + 1 (size >= 5000: 5000 x 5000), over 'ab' or 'a'..'z'.
seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
n, m = (5000, 5000) if S >= 5000 else (r.randint(1, S + 1), r.randint(1, S + 1))
k = 50 if S >= 5000 else r.randint(1, min(n, m))
abc = "ab" if seed % 2 == 0 or S >= 5000 else "abcdefghijklmnopqrstuvwxyz"
print(n, m, k)
for _ in range(n): print("".join(r.choice(abc) for _ in range(m)))
