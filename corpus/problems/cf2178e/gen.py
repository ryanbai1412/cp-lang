#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    steps = rng.randint(1, max(2, min(6, size)))
    arr = [1 << rng.randint(0, 3)]
    for _ in range(steps):
        if len(arr) > 32:
            break
        kind = rng.choice(["dup", "sum_r", "sum_l", "flat_r", "flat_l"])
        if kind == "dup":
            arr = arr + arr
        elif kind == "sum_r":
            arr = arr + [sum(arr)]
        elif kind == "sum_l":
            arr = [sum(arr)] + arr
        elif kind in ("flat_r", "flat_l") and any(x >= 2 for x in arr):
            flat = []
            for x in arr:
                if x >= 2:
                    flat.extend([x // 2, x // 2])
                else:
                    flat.append(x)
            arr = (arr + flat) if kind == "flat_r" else (flat + arr)
        else:
            arr = arr + arr
    print(len(arr))
    print(*arr)
