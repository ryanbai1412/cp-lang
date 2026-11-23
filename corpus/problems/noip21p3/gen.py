import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2]) if len(sys.argv) > 2 else 1
n = r.randint(1, 4 + 4 * S)
a = sorted(r.randint(1, min(600, 20 * S)) for _ in range(n))
print(n)
print(*a)
