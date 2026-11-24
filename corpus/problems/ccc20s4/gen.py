import sys, random
# gen.py seed size: one table of 1 <= N <= 4 size + 3 seats over A, B, C (some tests without C,
# some already in blocks with a few people out of place). --max: N = 1e6, blocks with 10% noise.
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 10**6 if MAX else r.randint(1, 4 * int(sys.argv[2]) + 3)
letters = "ABC" if MAX or r.random() < 0.7 else "AB"
if MAX or r.random() < 0.5:
    s = sorted(r.choice(letters) for _ in range(n))
    for _ in range(n // 10 if MAX else r.randint(0, n)):
        i, j = r.randrange(n), r.randrange(n); s[i], s[j] = s[j], s[i]
    k = r.randrange(n); s = s[k:] + s[:k]
else:
    s = [r.choice(letters) for _ in range(n)]
print("".join(s))
