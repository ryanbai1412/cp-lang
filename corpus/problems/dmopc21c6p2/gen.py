import sys, random
# statement: a 0/1 string of length 1 <= n <= 1e6, 1 <= k <= n.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 10 ** 6 if S >= 10 ** 6 else r.randint(1, max(1, min(10 ** 6, S)))
ones = r.random()
print(n, r.randint(1, n))
print("".join("1" if r.random() < ones else "0" for _ in range(n)))
