import sys, random
if sys.argv[1] == "--max":
    print(10**6, 1)  # N = 1e6 from the edge: the longest walk to the middle, ~5e5 moves printed
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: 1 <= N <= 1e6, 1 <= X <= N
MAX_N = 10**6
if S < 1000:
    n = r.randint(1, 4 * S + 3)
else:
    n = MAX_N - r.randrange(4)  # both parities and both residues of (n - 3) / 2 mod 4
x = r.choice([1, n, (n + 1) // 2, n // 2 + 1, r.randint(1, n)])
print(n, max(1, x))
