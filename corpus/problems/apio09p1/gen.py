import sys, random
if sys.argv[1:] == ["--max"]:
    g = random.Random(1)
    n = m = 1500
    k = 300
    print(n, m, k)
    for _ in range(n): print(*(g.randint(0, 500) for _ in range(m)))
    sys.exit()

# usage: gen.py SEED [SIZE]: an n x m field (each side <= SIZE where the squares fit) with k x k squares.
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([3, 8, 20, 60])
k = r.randint(1, max(1, S // 3))
# the three squares fit in a row, in a column, or as an L inside a 2k x 2k corner
need_n, need_m = r.choice([(3 * k, k), (k, 3 * k), (2 * k, 2 * k)])
n = r.randint(need_n, max(need_n, S)); m = r.randint(need_m, max(need_m, S))
top = r.choice([1, 5, 500])
print(n, m, k)
for _ in range(n): print(*(r.randint(0, top) for _ in range(m)))
