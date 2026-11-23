#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
# statement: t <= 1e4, 1 <= n, m, l <= 2e5, n <= l, m * l <= 2e5, a strictly increasing in 1..=l,
# sum of m * l <= 2e5
LIM = 200_000

def case(n, m, l):
    return (n, m, l, sorted(rng.sample(range(1, l + 1), n)))

if size < 1000:
    cases = []
    for _ in range(rng.randint(1, 5)):
        l = rng.randint(1, 25); cases.append(case(rng.randint(1, min(l, 10)), rng.randint(1, 10), l))
else:
    k = rng.randrange(5)
    if k == 0: cases = [case(LIM, 1, LIM)]
    elif k == 1: cases = [case(1, LIM, 1)]
    elif k == 2: cases = [case(446, 447, 447)]
    elif k == 3: cases = [case(rng.randint(1, 400), 400, 500)]
    else: cases = [case(rng.randint(1, 20), rng.randint(1, 10), 20) for _ in range(1000)]
print(len(cases))
for n, m, l, a in cases:
    print(n, m, l)
    print(*a)
