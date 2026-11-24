import sys, random
# seed size: a polynomial with a few planted integer roots in 1..=m times a random factor;
# --max: n = 100, m = 1e6, coefficients of 10^4 digits
if sys.argv[1] == "--max":
    r = random.Random(0); print(100, 10 ** 6)
    for i in range(101): print(r.choice("-") * r.randint(0, 1) + str(r.randint(1, 9)) + "".join(r.choice("0123456789") for _ in range(9999)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
m = r.randint(1, min(5 * S + 5, 10**6)); p = [r.choice([-3, -1, 1, 2, 5])]
for _ in range(r.randint(1, 4)):
    root = r.randint(1, m + 2) if r.random() < 0.8 else -r.randint(1, 5)
    q = [0] * (len(p) + 1)
    for i, c in enumerate(p): q[i + 1] += c; q[i] -= root * c
    p = q
for _ in range(r.randint(0, 2)):
    q = [0] * (len(p) + 1)
    for i, c in enumerate(p): q[i + 1] += c; q[i] += r.randint(1, 3) * c
    p = q
if p[-1] == 0: p[-1] = 1
print(len(p) - 1, m); print(*p, sep="\n")
