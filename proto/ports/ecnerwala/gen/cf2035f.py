import sys, random


def tree(r, n):
    shape = r.choice(["random", "path", "star", "caterpillar"])
    for i in range(2, n + 1):
        if shape == "path":
            p = i - 1
        elif shape == "star":
            p = 1
        elif shape == "caterpillar":
            p = max(1, i - r.choice([1, 2]))
        else:
            p = r.randint(1, i - 1)
        yield (i, p)


def case(r, n, hi):
    perm = list(range(1, n + 1))
    r.shuffle(perm)
    out = [f"{n} {r.randint(1, n)}", " ".join(str(r.randint(0, hi)) for _ in range(n))]
    for u, v in tree(r, n):
        out.append(f"{perm[u - 1]} {perm[v - 1]}" if r.random() < 0.5 else f"{perm[v - 1]} {perm[u - 1]}")
    return out


if sys.argv[1] == "--max":
    r = random.Random(1)
    print(1)
    print("\n".join(case(r, 2000, 10**9)))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 4)
print(t)
for _ in range(t):
    print("\n".join(case(r, r.randint(1, max(1, min(2000 // t, 1 + S))), r.choice([0, 1, 3, 10**9]))))
