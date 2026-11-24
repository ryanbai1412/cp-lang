import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", str(1 << 20)]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5 if big else r.randint(1, 5)
print(t)
for _ in range(t):
    n = S if big else r.randint(1, max(1, min(S, 1 << 20)))
    abc = "abcdefghijklmnopqrstuvwxyz"[:r.choice([1, 2, 3, 26])]
    unit = "".join(r.choice(abc) for _ in range(r.randint(1, max(1, n if r.random() < 0.3 else min(n, 6)))))
    s = (unit * (n // len(unit) + 1))[:n]
    s = "".join(c if r.random() < 0.97 else r.choice(abc) for c in s)
    print(s)
