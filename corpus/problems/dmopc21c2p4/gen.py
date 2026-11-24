import sys, random
# statement: 1 <= K <= N <= 1e6, heights h_i <= 1e9.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
n = 10 ** 6 if big else r.randint(1, max(1, min(10 ** 6, S)))
k = r.randint(1, n) if not big else r.choice([1, 3, 1000])
H = r.choice([2, 3, 10 ** 9])
h = []
while len(h) < n:
    h += [r.randint(0, H)] * r.randint(1, max(1, 2 * k + 3 if not big else 50))
print(n, k)
print(*h[:n])
