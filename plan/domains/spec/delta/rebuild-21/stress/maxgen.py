#!/usr/bin/env python3
"""Maximum-size inputs for timing the C++ build: maxgen.py NAME > NAME.max.in"""
import random, sys
rng = random.Random(21); name = sys.argv[1]; o = []
w = lambda *r: o.append(" ".join(map(str, r)))
if name == "01-1974E":
    w(2)
    for _ in range(2):
        w(50, 10**8)
        for _ in range(50): w(rng.randint(0, 10**8), 1000)
elif name == "01-1974E-rev":
    w(1); w(50, 10**8, 10)
    for _ in range(50): w(rng.randint(0, 10**8), 200)
elif name in ("02-1969C", "02-1969C-rev"):
    n = 300000; w(1); w(n, 10) if name == "02-1969C" else w(n, 5, 5); w(*[rng.randint(1, 10**9) for _ in range(n)])
elif name in ("03-1968G1", "03-1968G1-rev"):
    n = 200000; w(1); w(n, 2, 2) if name == "03-1968G1" else w(n, 2, 22)
    w("".join("a" if rng.random() < .95 else "b" for _ in range(n)))
elif name == "04-1954E":
    n = 100000; w(n); w(*[rng.randint(1, 10**5) for _ in range(n)])
elif name == "05-1976D":
    s = []; bal = 0; op = 100000
    while op or bal:
        if op and (bal == 0 or rng.random() < .5): s.append("("); op -= 1; bal += 1
        else: s.append(")"); bal -= 1
    w(1); w("".join(s))
elif name == "06-1955G":
    w(20)
    for _ in range(20):
        w(100, 100)
        for _ in range(100): w(*[720720] * 100)
elif name == "07-1978E":
    n = 200000; w(1); w(n); w("".join(rng.choice("0001") for _ in range(n))); w("".join(rng.choice("0111") for _ in range(n)))
    w(n)
    for _ in range(n):
        l = rng.randint(1, n); w(l, rng.randint(l, n))
elif name == "08-1957C":
    w(1); w(300000, 0)
elif name == "09-1730B":
    w(2)
    for _ in range(2):
        n = 100000; w(n); w(*[rng.randint(0, 10**8) for _ in range(n)]); w(*[rng.randint(0, 10**8) for _ in range(n)])
elif name == "10-1543C":
    w(10)
    for _ in range(10): w("0.4000 0.4000 0.2000 0.1000")
print("\n".join(o))
