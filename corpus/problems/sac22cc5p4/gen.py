import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
# `seed size`: n, q <= size + 1 (size >= 1e5 gives 1e5 each over the full range).
seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed); N = 10 ** 5
n, q = (N, N) if S >= N else (r.randint(1, S + 1), r.randint(1, S + 1))
w = 10 ** 9 if S >= N or seed % 3 == 0 else r.choice([5, 20])
def iv(span):
    l = r.randint(-w, w); return l, min(w, l + r.randint(0, span))
print(n, q)
for _ in range(n): print(*iv(w // 50 + 3))
for _ in range(q): print(*iv(w // 20 + 3))
