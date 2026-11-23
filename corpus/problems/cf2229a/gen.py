# Usage: gen.py seed size. Valid under the statement: t <= 100, 2 <= n <= 1000,
# 1 <= a_i <= 1000, sum of n <= 1000. size >= 1000 emits a max-size shape picked by seed.
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
A = 1000
if S >= 1000:
    shape = int(sys.argv[1]) % 5
    if shape == 0:
        tests = [[r.randint(1, A) for _ in range(1000)]]
    elif shape == 1:
        tests = [[1] * 999 + [A]]
    elif shape == 2:
        tests = [[r.choice((1, A)) for _ in range(10)] for _ in range(100)]
    elif shape == 3:
        tests = [[A] * 1000]
    else:
        tests = [[r.randint(1, A) for _ in range(2)] for _ in range(99)] + [[r.randint(1, A) for _ in range(802)]]
else:
    t = r.randint(1, 5); tests = []
    for _ in range(t):
        n = r.randint(2, 2 + 3 * S); hi = r.choice((3, 10, A))
        tests.append([r.randint(1, hi) for _ in range(n)])
print(len(tests))
for a in tests:
    print(len(a)); print(*a)
