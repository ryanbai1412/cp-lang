import sys, random
# gen.py seed size: 1 <= M <= 100 memory cells, 1 <= N <= min(1000, 20 size) words in 0..=1000 drawn
# from a small or a full vocabulary. --max: M = 100, N = 1000, cycling 101 words (every lookup misses).
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    m, n = 100, 1000; words = [(i % 101) * 9 for i in range(n)]
else:
    m = r.randint(1, 100); n = r.randint(1, min(1000, 20 * int(sys.argv[2])))
    v = r.choice([3, m + 1, 1000]); words = [r.randint(0, min(v, 1000)) for _ in range(n)]
print(m, n); print(*words)
