import sys, random
if sys.argv[1] == "--max":  # sum n = 4e5: one n = 4e5 random RBS, k = n / 4
    rng = random.Random(0); h = 200000; s = []; o = c = 0
    while c < h:
        if o < h and (o == c or rng.random() < 0.5): s.append("("); o += 1
        else: s.append(")"); c += 1
    print(1); print(2 * h, h // 2); print("".join(s)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: t <= 1e4, 2 <= n <= 4e5 (n even, s an RBS), 0 <= k <= n/2, sum of n <= 4e5
MAX_N, MAX_T = 400_000, 10_000

def rbs(h, p):
    s = []; o = c = 0
    while c < h:
        if o < h and (o == c or r.random() < p): s.append("("); o += 1
        else: s.append(")"); c += 1
    return "".join(s)

if S < 1000:
    hs = [r.randint(1, 3 * S) for _ in range(5)]
else:
    hs = [MAX_N // 2] if r.randrange(3) else [MAX_N // 2 // MAX_T] * MAX_T
print(len(hs))
for h in hs:
    shape = r.randrange(4)
    s = "(" * h + ")" * h if shape == 0 and S >= 1000 else "()" * h if shape == 1 and S >= 1000 else rbs(h, r.choice([0.3, 0.5, 0.7, 0.9]))
    k = r.choice([0, h, r.randint(0, h), r.randint(0, min(h, 50))])
    print(2 * h, k); print(s)
