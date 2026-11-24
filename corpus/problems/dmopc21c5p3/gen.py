import sys, random
# statement: t <= 1e5 tests, 1 <= n <= 1e5, sum of n <= 5e6.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 100000:
    ns = [100000 - i for i in range(47)] + [11, 36, 311]
else:
    ns = [r.choice([11, 36, 311, r.randint(1, max(1, 3 * S))]) for _ in range(r.randint(1, 6))]
print(len(ns))
for n in ns:
    print(n)
