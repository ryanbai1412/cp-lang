"""wtf26b generator: `seed size`; size >= 50 gives N = 50 (all N, random, all 1, ramps)."""
import random
import sys

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S >= 50:
    n = 50
    mode = int(sys.argv[1]) % 5
    if mode == 0:
        a = [n] * n
    elif mode == 1:
        a = [r.randint(1, n) for _ in range(n)]
    elif mode == 2:
        a = [1] * n
    elif mode == 3:
        a = [i % n + 1 for i in range(n)]
    else:
        a = [r.randint(n // 2, n) for _ in range(n)]
else:
    n = r.randint(2, min(S + 2, 8))
    a = [r.randint(1, n) for _ in range(n)]
print(n)
print(*a)
