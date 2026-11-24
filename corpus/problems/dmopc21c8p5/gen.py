import sys, random
# statement: 3 <= N <= 1e9, 2 <= K <= 3000, costs 1 <= a_i <= 1e9.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 9
n = 10 ** 9 if big else r.randint(3, max(3, r.choice([S, 10 * S, 10 ** 9])))
k = 3000 if big else r.randint(2, max(2, min(S, 12)))
top = 10 ** 9 if big or r.random() < 0.5 else 20
print(n, k)
print(*(r.randint(1, top) for _ in range(k)))
