import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 50, l = 1e4, small varied radii so every (blocks, cells) state stays reachable
    r = random.Random(0)
    print(50, 10 ** 4)
    print(*[r.randint(1, 150) for _ in range(50)])
    sys.exit()
# usage: gen.py SEED [MAXL]; COCI 2021/22 C2 Magneti: 1 <= n <= 50, n <= l <= 1e4, 1 <= r_i <= l
r = random.Random(int(sys.argv[1])); L = int(sys.argv[2]) if len(sys.argv) > 2 else 10**4
n = r.randint(1, min(50, L))
rs = [r.randint(1, r.choice([1, 3, 10, L])) for _ in range(n)]
tight = 1 + sum(sorted(rs)[1:])  # about the shortest row that fits them all
l = r.choice([r.randint(n, L), min(L, max(n, tight + r.randint(-5, 50)))])
print(n, l)
print(*[min(x, l) for x in rs])
