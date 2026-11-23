import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, max(2, min(S, 100000)))
cmax = r.choice([1, 10, 10000] + ([10**6] if n <= 1000 else []))
protos = r.choice([[0, 1, 2], [0], [1], [2], [0, 1], [0, 2], [1, 2]])
hosts = r.choice(["any", "last", "star", "near"])
def host(i):
    if hosts == "last": return i - 1
    if hosts == "star": return 0
    if hosts == "near": return r.randint(max(0, i - 3), i - 1)
    return r.randrange(i)
print(n)
print(*[r.randint(1, cmax) for _ in range(n)])
print(*[f"{host(i)} {r.choice(protos)}" for i in range(1, n)])
