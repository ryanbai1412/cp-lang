import sys, random

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
