import sys, random
# statement: 1 <= k <= n <= 500, s over ( ) * ?.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 500 if S >= 500 else r.randint(1, max(1, min(500, S)))
k = r.randint(1, n)
w = r.choice([[1, 1, 1, 6], [3, 3, 2, 2], [1, 1, 3, 5]])
s = r.choices("()*?", weights=w, k=n)
if S >= 500: s = ["?"] * n
print(n, k); print("".join(s))
