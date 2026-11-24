import sys, random
if sys.argv[1:] == ["--max"]:
    # n = 1e5 on a path (the deepest tree), m = 5e4 ministers of two far-apart cities each
    # (sum S = 1e5), k = 1 so every wanted track is printed
    g = random.Random(1)
    n, m = 10**5, 5 * 10**4
    lab = list(range(1, n + 1)); g.shuffle(lab)
    print(n, m, 1)
    for i in range(1, n): print(lab[i - 1], lab[i])
    for _ in range(m): print(2, lab[g.randrange(n // 2)], lab[n // 2 + g.randrange(n // 2)])
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 3); m = r.randint(1, S + 1); print(n, m, r.randint(1, m))
for i in range(2, n + 1): print(r.randint(1, i - 1), i)
for _ in range(m):
    s = r.randint(2, n); print(s, *r.sample(range(1, n + 1), s))
