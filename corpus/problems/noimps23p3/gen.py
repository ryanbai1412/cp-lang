import sys, random
# seed size: n, k <= size + 1 and m <= size, a random, chain (property B), star or deep tree,
# values all 1 (C), few distinct, or up to 1e5; events add or fire (none fired for A); sid is the
# first subtask whose bounds and properties the case meets. --max: sid 15, n = k = m = 1e5, a
# deep random tree (parent among the 100 before), random values, adds and fires half and half
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
B5 = 10**5
n = B5 if mx else r.randint(1, min(B5, S + 1))
k = B5 if mx else r.randint(1, min(B5, S + 1))
m = B5 if mx else r.randint(0, min(B5, S))
kind = "deep" if mx else r.choice(["rand", "chain", "star", "deep"])
vk = "big" if mx else r.choice(["one", "few", "big"])
noFire = False if mx else r.random() < 0.3
def val(): return 1 if vk == "one" else r.randint(1, 3) if vk == "few" else r.randint(1, B5)
p = []
for i in range(2, n + 1):
    p.append(i - 1 if kind == "chain" else 1 if kind == "star" else r.randint(max(1, i - 100), i - 1) if kind == "deep" else r.randint(1, i - 1))
emp = [(r.randint(1, n), val()) for _ in range(k)]
alive = list(range(1, k + 1)); tot = k; ev = []
for _ in range(m):
    if alive and not noFire and r.random() < 0.5:
        j = r.randrange(len(alive)); alive[j], alive[-1] = alive[-1], alive[j]
        ev.append((2, alive.pop()))
    else:
        tot += 1; alive.append(tot); ev.append((1, r.randint(1, n), val()))
A = all(e[0] == 1 for e in ev); Bp = all(p[i] == i + 1 for i in range(n - 1))
C = all(v == 1 for _, v in emp) and all(e[2] == 1 for e in ev if e[0] == 1)
rows = [(6, 6, 6, ""), (9, 6, 6, ""), (16, 66, 66, ""), (66, 66, 0, ""), (2333, 2333, 0, ""),
        (B5, B5, 0, "B"), (B5, B5, 0, ""), (2333, 2333, 2333, "A"), (B5, B5, B5, "AB"),
        (B5, B5, B5, "A"), (2333, 2333, 2333, ""), (B5, B5, B5, "C"), (B5, B5, B5, "B"),
        (66666, 66666, 66666, ""), (B5, B5, B5, "")]
have = {"A": A, "B": Bp, "C": C}
sid = next(i + 1 for i, (N, K, M, pr) in enumerate(rows) if n <= N and k <= K and m <= M and all(have[c] for c in pr))
print(sid); print(n, k, m); print(*p)
for x, v in emp: print(x, v)
for e in ev: print(*e)
