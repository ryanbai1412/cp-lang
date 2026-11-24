"""gen.py SEED SIZE | --max: NOI 2004 One-Way Streets. m <= 10 horizontal and n <= 100 vertical
streets, flip costs 0..1e4, 1 <= k <= 100 requests between intersections. Most inputs hide
an orientation that meets every request, so the answer is usually possible and the search
has to find the cheapest one; the rest draw requests freely. --max is m = 10, n = 100,
k = 100 with a hidden orientation and costs over the full range."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
m = 10 if MAX else r.randint(1, min(10, 1 + S // 2))
n = 100 if MAX else r.randint(1, min(100, 1 + S))
k = 100 if MAX else r.randint(1, min(100, 1 + S))
w = 10_000 if MAX or r.random() < 0.5 else r.randint(0, 3)
h = "".join(r.choice("EW") for _ in range(m))
v = "".join(r.choice("NS") for _ in range(n))
hidden = MAX or r.random() < 0.7
hh = "".join(r.choice("EW") for _ in range(m))
vv = "".join(r.choice("NS") for _ in range(n))
reqs = []
while len(reqs) < k:
    x1, y1 = r.randint(1, m), r.randint(1, n)
    if not hidden:
        reqs.append((x1, y1, r.randint(1, m), r.randint(1, n)))
        continue
    x, y, path = x1, y1, r.randint(0, m + n)
    for _ in range(path):
        moves = [(x, y + (1 if hh[x - 1] == "E" else -1)), (x + (1 if vv[y - 1] == "S" else -1), y)]
        moves = [p for p in moves if 1 <= p[0] <= m and 1 <= p[1] <= n
                 and (p[0] - x1) * (p[0] - x) >= 0 and (p[1] - y1) * (p[1] - y) >= 0]
        if not moves:
            break
        x, y = r.choice(moves)
    reqs.append((x1, y1, x, y))
print(m, n)
print(h)
print(v)
print(*(r.randint(0, w) for _ in range(m)))
print(*(r.randint(0, w) for _ in range(n)))
print(k)
sys.stdout.write("".join(" ".join(map(str, q)) + "\n" for q in reqs))
