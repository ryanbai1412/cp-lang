import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 3; print(t)
for _ in range(t):
    n = r.randint(2, S + 4); print(n)
    print("".join(r.choice("01?") for _ in range(n - 1)))
    ch = [[] for _ in range(n)]
    for v in range(2, n + 1):
        ch[r.randint(1, v - 1) - 1].append(v)
    for c in ch:
        print(len(c), *c)
