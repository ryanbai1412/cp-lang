import sys, random
# statement: n <= 1e6 numbers 1 <= a_i <= 1e9, 1 <= m <= 2e9.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
n = 10 ** 6 if big else r.randint(1, max(1, min(S, 8)))
top = 10 ** 9 if big else r.choice([5, 20, 10 ** 9])
a = [r.randint(1, top) for _ in range(n)]
m = 10 ** 9 if big else r.randint(1, 2 * top)
print(n, m)
print(*a)
