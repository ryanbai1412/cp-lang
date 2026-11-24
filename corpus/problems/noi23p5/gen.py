import sys, random
# seed size: t <= 5 strings of n <= size letters (random over a/b, adjacent letters distinct, or a few
# letters), q <= size queries (i, r) with i + 2r - 1 <= n; --max: t = 5, n = q = 1e5, one string per
# shape (all a, random a/b, ab-periodic, adjacent-distinct, 26 letters), r as large as i allows
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
shapes = ["a", "ab", "period", "distinct", "any"]
t = 5 if mx else r.randint(1, 5)
print(25, t)
for k in range(t):
    n = 10**5 if mx else r.randint(2, max(2, min(10**5, S)))
    q = 10**5 if mx else r.randint(1, max(1, min(10**5, S)))
    shape = shapes[k] if mx else r.choice(shapes)
    if shape == "a":
        s = "a" * n
    elif shape == "ab":
        s = "".join(r.choice("ab") for _ in range(n))
    elif shape == "period":
        p = "".join(r.choice("ab") for _ in range(r.randint(1, 4)))
        s = (p * n)[:n]
    elif shape == "distinct":
        s = [r.choice("abc")]
        while len(s) < n:
            s.append(r.choice([c for c in "abc" if c != s[-1]]))
        s = "".join(s)
    else:
        s = "".join(r.choice("abcdefghijklmnopqrstuvwxyz"[:r.choice([2, 3, 26])]) for _ in range(n))
    print(n, q); print(s)
    for _ in range(q):
        i = r.randint(1, n - 1)
        hi = (n - i + 1) // 2
        print(i, hi if mx or r.random() < 0.3 else r.randint(1, hi))
