import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: t <= 1e4, 2 <= n <= 2e5, 0 <= q <= 2e5, 0 <= a_i, x < 16, sums of n and q <= 2e5
MAX, MAX_T = 200_000, 10_000
if S < 1000:
    cases = [(r.randint(2, 3 * S + 1), r.randint(0, 3 * S)) for _ in range(4)]
else:
    cases = [(MAX, MAX)] if r.randrange(3) else [(MAX // MAX_T, MAX // MAX_T)] * MAX_T
print(len(cases))
for n, q in cases:
    print(n, q); print(*[r.randrange(16) for _ in range(n)])
    for _ in range(q): print(r.randint(1, n), r.randrange(16))
