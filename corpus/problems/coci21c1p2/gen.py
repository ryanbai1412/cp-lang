import sys, random
# statement: 1 <= k < n <= 350, a row of C and P with at least 2k - 1 C.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "350"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 350 if S >= 350 else r.randint(2, max(2, min(S, 350)))
k = r.randint(1, min(n - 1, (n + 1) // 2))
c = r.randint(2 * k - 1, n)
s = ["C"] * c + ["P"] * (n - c)
r.shuffle(s)
print(n, k)
print("".join(s))
