import random, sys
n, q, seed = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
lazy = len(sys.argv) > 4 and sys.argv[4] == "lazy"
random.seed(seed)
par = [0, 0] + [random.randint(1, v - 1) for v in range(2, n + 1)]
out = [f"{n}", " ".join(str(random.randint(-10**9, 10**9)) for _ in range(n)), " ".join(map(str, par[2:])), f"{q}"]
for _ in range(q):
    t = random.randint(0, 2)
    if t == 0:
        if lazy:
            out.append(f"0 {random.randint(1, n)} {random.randint(1, n)} {random.randint(-10**4, 10**4)}")
        else:
            out.append(f"0 {random.randint(1, n)} {random.randint(-10**9, 10**9)}")
    elif t == 1:
        v = random.randint(2, n)
        p = random.randint(1, v - 1)
        out.append(f"1 {v} {par[v]} {p}")
        par[v] = p
    else:
        out.append(f"2 {random.randint(1, n)} {random.randint(1, n)}")
print("\n".join(out))
