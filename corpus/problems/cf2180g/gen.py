#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # 40 inserts make L = 2^40 - 1, then erases alternate with queries for the rest
    rng = random.Random(1)
    q, p = 10**6, 10**9 + 7
    out, L = [], 0
    for i in range(q):
        op = 2 if i < 40 else (3 if i % 2 or i == q - 1 else 1)
        if op == 1:
            L -= 1
            out.append("1")
        elif op == 2:
            L = 2 * L + 1
            out.append(f"2 {rng.randint(1, 10**9)}")
        else:
            assert L % p
            out.append("3")
    print(1)
    print(q)
    print("\n".join(out))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    q = rng.randint(2, max(4, size * 6))
    print(q)
    cur_len = 0
    for _ in range(q):
        if cur_len == 0:
            op = 2
        elif cur_len == 1:
            op = rng.choice([2, 3])
        else:
            op = rng.choice([1, 2, 3])
        if op == 1:
            cur_len -= 1
            print(1)
        elif op == 2:
            cur_len = 2 * cur_len + 1
            print(2, rng.randint(1, 1000000006))
        else:
            print(3)
