import sys, random
# statement: t <= 200 tests, 1 <= n <= 50, 0 <= f_i <= 1e16.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
t = 200 if big else r.randint(1, 5)
print(t)
for _ in range(t):
    n = 50 if big else r.randint(1, max(1, min(S, 8)))
    top = r.choice([0, 3, 10 ** 16]) if not big else r.choice([0, 1, 10 ** 16])
    print(n)
    print(*(r.randint(0, top) for _ in range(n)))
