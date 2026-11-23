#!/usr/bin/env python3
"""Valid CF 2262A2 input: a comes from a hidden set A of {0..n-1}; n, sum n <= 1e5."""
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])


def case(n, p):
    inA = [r.random() < p for _ in range(n)]
    pre = [0]
    for b in inA:
        pre.append(pre[-1] + b)
    cnt = lambda lo, hi: pre[min(hi, n)] - pre[min(lo, n)]
    a = []
    for k in range(1, n + 1):
        j = 0
        while cnt(j * k, j * k + k) > 0:
            j += 1
        a.append(j)
    return n, a


if size >= 1000:
    mode = r.randrange(5)
    if mode == 0:
        tests = [case(100000, 0.5)]
    elif mode == 1:
        tests = [case(100000, 1.0)]
    elif mode == 2:
        tests = [case(100000, 0.97)]
    elif mode == 3:
        tests = [case(100000, 0.0)]
    else:
        tests = [case(10, r.random()) for _ in range(10000)]
else:
    tests = [case(r.randint(1, 4 * size + 8), r.choice([0.3, 0.6, 0.9, 1.0])) for _ in range(5)]
print(len(tests))
for n, a in tests:
    print(n)
    print(*a)
