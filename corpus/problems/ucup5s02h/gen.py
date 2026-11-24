#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # statement max: sum n <= 5e5 with n <= 1e5, so five cases of n = 1e5, q <= 15000 and
    # distinct x <= 2e5. Shapes that keep Boruvka busy: random, all-equal q on consecutive x,
    # strictly decreasing q (deep hulls), alternating 1/15000, and random with 10% pins.
    rng = random.Random(0)
    n, X, Q = 10**5, 2 * 10**5, 15000
    out = ["5"]
    for case in range(5):
        if case == 1: xs = list(range(1, n + 1))
        elif case == 2: xs = list(range(2, X + 1, 2))
        else: xs = sorted(rng.sample(range(1, X + 1), n))
        if case == 0: qs = [rng.randint(1, Q) for _ in range(n)]
        elif case == 1: qs = [Q] * n
        elif case == 2: qs = [Q - i * (Q - 1) // (n - 1) for i in range(n)]
        elif case == 3: qs = [1 if i % 2 else Q for i in range(n)]
        else: qs = [rng.randint(1, Q) for _ in range(n)]
        pins = ["?"] * n
        if case == 4:
            for i in rng.sample(range(1, n), n // 10): pins[i] = rng.choice("AB")
        order = list(range(n)); rng.shuffle(order)
        out.append(str(n))
        out += [f"{qs[i]} {xs[i]} {pins[i]}" for i in order]
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, 14)
    print(n)
    xs = rng.sample(range(-30, 31), n)
    pin = rng.random() < 0.5
    for x in xs:
        c = rng.choice("AB?") if pin else "?"
        print(rng.randint(1, 20), x, c)
