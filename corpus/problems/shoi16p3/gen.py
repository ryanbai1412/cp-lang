"""gen.py seed size: SHOI 2016 Grade Comparison. m, n <= 100, top <= 1e9, B's rank in 1..=m,
and k students no higher than B everywhere, so k <= m - rank in every course. --max: m = n = 100,
top = 1e9 and B near the top, so `below` is largest in every course."""
import random, sys
mx = sys.argv[1:] == ["--max"]
r = random.Random(1 if mx else int(sys.argv[1]))
if mx:
    m, n = 100, 100
    top = [10 ** 9 - i for i in range(n)]
    rank = [r.randint(1, 3) for _ in range(n)]
else:
    size = int(sys.argv[2])
    m, n = r.randint(1, max(1, min(size, 100))), r.randint(1, max(1, min(size, 100)))
    top = [r.randint(1, r.choice([5, 10 ** 9])) for _ in range(n)]
    rank = [r.randint(1, m) for _ in range(n)]
k = r.randint(0, min(m - x for x in rank))
print(m, n, k); print(*top); print(*rank)
