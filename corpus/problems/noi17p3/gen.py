import sys, random
# seed size: N <= size^3 (at most 1e9), K <= 10 size (at most 1000), 1 <= x < y < 998244353;
# --max: N = 1e9, K = 1000 (the recurrence of order 1001 powered 30 times)
if sys.argv[1] == "--max":
    print(10**9, 1000, 1, 2); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
y = r.choice([r.randint(2, 10), r.randint(2, 998244352)]); x = r.randint(1, y - 1)
print(r.randint(1, min(10**9, S ** 3)), r.randint(1, min(1000, 10 * S)), x, y)
