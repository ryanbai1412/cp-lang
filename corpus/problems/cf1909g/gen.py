#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    # m = 1e7, delta = m - n = 7207200 (288 divisors), all one letter: every divisor is a period
    # and every window start counts, so the answer is large
    m = 10**7
    n = m - 7_207_200
    print(n, m)
    print("a" * n)
    print("a" * m)
    sys.exit()

seed = int(sys.argv[1])
size = int(sys.argv[2])
rng = random.Random(seed)

if size >= 10000:
    # Construct a valid pumping instance of size ~size
    n = size // 2
    m = size
    # Choose a small repeating pattern y
    pat_len = rng.choice([1, 2, 3, 6, 12])
    y = "".join(rng.choice("ab") for _ in range(pat_len))
    x_len = rng.randint(0, n // 4)
    x = "".join(rng.choice("abc") for _ in range(x_len))
    z_len = rng.randint(0, n // 4)
    z = "".join(rng.choice("abc") for _ in range(z_len))
    mid_s = n - x_len - z_len
    if mid_s < pat_len:
        mid_s = pat_len
    k_extra = (m - n) // pat_len
    s_mid = (y * ((mid_s // pat_len) + 2))[:mid_s]
    s = x + s_mid + z
    t = x + (y * k_extra) + s_mid + z
    print(len(s), len(t))
    print(s)
    print(t)
else:
    mode = rng.randint(0, 3)
    if mode == 0:
        # random strings (often 0)
        n = rng.randint(1, max(1, size))
        m = rng.randint(n + 1, max(n + 1, size + 2))
        s = "".join(rng.choice("ab") for _ in range(n))
        t = "".join(rng.choice("ab") for _ in range(m))
    else:
        # constructed valid instance
        pat_len = rng.randint(1, max(1, min(6, size)))
        y = "".join(rng.choice("abc") for _ in range(pat_len))
        x_len = rng.randint(0, max(0, size // 3))
        z_len = rng.randint(0, max(0, size // 3))
        x = "".join(rng.choice("abc") for _ in range(x_len))
        z = "".join(rng.choice("abc") for _ in range(z_len))
        k1 = rng.randint(1, 3)
        k2 = k1 + rng.randint(1, 4)
        s = x + y * k1 + z
        t = x + y * k2 + z
        if mode == 3 and len(s) >= 2:
            # occasionally perturb one char
            pos = rng.randint(0, len(s) - 1)
            s = s[:pos] + rng.choice("abc") + s[pos + 1:]
    print(len(s), len(t))
    print(s)
    print(t)
