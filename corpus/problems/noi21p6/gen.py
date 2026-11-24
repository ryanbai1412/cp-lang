import sys, random
# seed size: n <= min(32, size + 1) cells, m <= size + 1 robots, programs of length <= 2 size + 1
# with a random share of R; --max: n = 32, m = 1000 programs of length 100 holding 0 to 15 R, so
# both the narrow-span enumeration and the wide-span DP see live robots
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 32 if mx else r.randint(1, min(32, S + 1))
m = 1000 if mx else r.randint(1, min(1000, S + 1))
print(n, m)
for i in range(m):
    L = 100 if mx else r.randint(1, min(100, 2 * S + 1))
    k = (i % 16) if mx else r.randint(0, min(L, r.choice([0, 1, 2, n, L])))
    s = [r.choice("01*") for _ in range(L)]
    for j in r.sample(range(L), min(k, L)): s[j] = "R"
    print("".join(s))
