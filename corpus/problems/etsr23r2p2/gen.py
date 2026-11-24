import sys, random
# Multitest: a digit string and a target. Half the targets come from a random placement of signs,
# so the answer is not always -1.
def case(r, n):
    s = ''.join(r.choice('0123456789' if r.random() < 0.7 else '0112') for _ in range(n))
    if r.random() < 0.5:
        expr = s[0] + ''.join(r.choice(['', '+', '*']) + ch for ch in s[1:])
        try:
            k = eval('+'.join('*'.join(str(int(f)) for f in t.split('*')) for t in expr.split('+')))
        except Exception:
            k = 0
        if k > 254: k = r.randint(0, 254)
    else:
        k = r.randint(0, 254)
    return s, k
if sys.argv[1] == "--max":
    r = random.Random(1)
    tests = [case(r, 24) for _ in range(3)]
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    tests = [case(r, r.randint(1, max(1, min(S, 8)))) for _ in range(r.randint(1, 3))]
print(len(tests))
for s, k in tests:
    print(s, k)
