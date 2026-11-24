import sys, random
# seed size: T <= 5 permutations of n <= size + 1, either random or good (each value a new
# maximum or the smallest unused) with a late change, so the walk goes deep; --max: T = 5 with
# sum n = 2e6 (n = 6e5, 6e5, 6e5, 1e5, 1e5)
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1]))
ns = [600000, 600000, 600000, 100000, 100000] if mx else \
    [r.randint(1, int(sys.argv[2]) + 1) for _ in range(r.randint(1, 5))]
print(len(ns))
for n in ns:
    if r.random() < 0.4:
        p = list(range(1, n + 1)); r.shuffle(p)
    else:
        p, used, small, top = [], [False] * (n + 2), 1, 0
        for _ in range(n):
            if top < n and (r.random() < 0.5 or small > top):
                top = r.randint(top + 1, min(n, top + 3)); v = top
            else:
                v = small
            p.append(v); used[v] = True
            while small <= n and used[small]: small += 1
            if small > n: break
        if r.random() < 0.5 and n > 1:
            i = r.randrange(n - 1); p[i], p[i + 1] = p[i + 1], p[i]
    print(n); print(*p)
