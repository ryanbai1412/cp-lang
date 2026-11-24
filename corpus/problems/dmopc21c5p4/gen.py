import sys, random
# statement: T tests, 1 <= N <= 1e6, sum of N <= 1e6.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 10 ** 6:
    ns = [10 ** 6 - 20] + list(range(1, 21))
else:
    ns = [r.randint(1, max(1, 2 * S)) for _ in range(r.randint(1, 8))]
print(len(ns))
for n in ns:
    print(n)
