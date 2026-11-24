import sys, random
# statement: 1 <= N <= 1e6, 0 <= K <= 1e18.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 10 ** 6:
    n = 10 ** 6; k = r.randint(10 ** 17, 10 ** 18)
else:
    n = r.randint(1, max(1, min(S, 12)))
    most = sum((i // 2) * (i - i // 2 - 1) for i in range(1, n + 1))
    k = r.randint(0, most + 1)
print(n, k)
