import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 5)
print(t)
for _ in range(t):
    n = r.randint(1, 3 * S * S)
    mode = r.randint(0, 3)
    if mode == 0:
        # Random valid-looking non-increasing array in [n-i, n]
        a = []
        hi = n
        for i in range(1, n + 1):
            lo = n - i
            v = r.randint(lo, hi)
            a.append(v)
            hi = v
    elif mode == 1:
        # All equal to n
        a = [n] * n
    elif mode == 2:
        # Strictly decreasing from n down to 0 or 1
        start = r.randint(n - 1, n)
        a = [max(0, start - i) for i in range(n)]
    else:
        # Arbitrary (mostly invalid)
        hi = r.choice([n, n + 2, 10**9])
        a = [r.randint(0, hi) for _ in range(n)]
    print(n)
    print(*a)
