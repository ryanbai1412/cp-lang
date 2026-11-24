import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e6: many short runs with sporadic tall ones keep both staircases busy
    rng = random.Random(1)
    n, parts, c = 2_000_000, [], 0
    while n > 0:
        ln = min(n, rng.choice((1, 1, 2, 3, rng.randint(1, 50))))
        parts.append(str(c) * ln)
        n -= ln
        c ^= 1
    print(1)
    print(2_000_000)
    print("".join(parts))
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = 6
print(t)
for _ in range(t):
    n = r.randint(3, 8 * S + 5)
    mode = r.choice(["rand", "all0", "all1", "alt", "runs"])
    if mode == "all0":
        s = "0" * n
    elif mode == "all1":
        s = "1" * n
    elif mode == "alt":
        s = "".join(str((i + r.randint(0, 1)) % 2) for i in range(n))
    elif mode == "runs":
        out = []
        rem = n
        c = r.choice(["0", "1"])
        while rem > 0:
            ln = r.randint(1, min(rem, max(2, n // 3)))
            out.append(c * ln)
            rem -= ln
            c = "1" if c == "0" else "0"
        s = "".join(out)
    else:
        s = "".join(r.choice(["0", "1"]) for _ in range(n))
    print(n)
    print(s)
