#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test with n = sum n = 4e5: a deep, thin tree (parent among the last three nodes, so
    # depth ~2e5) with random weights 1..n, shuffled labels apart from the root 1
    r = random.Random(1)
    n = 400_000
    lab = [1] + r.sample(range(2, n + 1), n - 1)
    out = ["1", str(n), " ".join(str(r.randint(1, n)) for _ in range(n))]
    for i in range(1, n):
        p = r.randint(max(0, i - 3), i - 1)
        out.append(f"{lab[p]} {lab[i]}" if r.random() < 0.5 else f"{lab[i]} {lab[p]}")
    print("\n".join(out))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
T = 5 if S == 0 else 4
print(T)
for _ in range(T):
    n = r.randint(1, 10 if S == 0 else min(500, 12 * S + 10))
    mode = r.choice(["rand", "distinct", "small_w", "line"])
    if mode == "distinct":
        w = list(range(1, n + 1))
        r.shuffle(w)
    elif mode == "small_w":
        k = r.randint(1, min(n, 4))
        w = [r.randint(1, k) for _ in range(n)]
    else:
        w = [r.randint(1, n) for _ in range(n)]
    print(n)
    print(*w)
    for i in range(2, n + 1):
        p = i - 1 if mode == "line" else r.randint(1, i - 1)
        if r.random() < 0.5:
            print(p, i)
        else:
            print(i, p)
