import sys, random
# seed size: n <= size + 1 packages, package i > 0 depending on a random earlier one (labels
# shuffled, 0 the root), q <= size + 1 random commands; --max: n = q = 1e5 on a long path
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 10**5 if mx else int(sys.argv[2])
n = 10**5 if mx else r.randint(1, min(10**5, S + 1))
q = 10**5 if mx else r.randint(1, min(10**5, S + 1))
lab = [0] + r.sample(range(1, n), n - 1)
par = [0] * n
for i in range(1, n):
    j = i - 1 if mx or r.random() < 0.3 else r.randrange(i)
    par[lab[i]] = lab[j]
print(n); print(*par[1:])
print(q)
for _ in range(q): print(r.choice(["install", "uninstall"]), r.randrange(n))
