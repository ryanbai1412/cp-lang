import sys, random
# gen.py seed size: t tests, n <= min(3 size + 2, 100) towers of heights 1..=1000.
# No arguments, or size >= 100, gives t = 500 tests of n = 100.
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = int(sys.argv[2]) if len(sys.argv) > 2 else 100
r = random.Random(seed)
full = S >= 100
t = 500 if full else 5; print(t)
for _ in range(t):
    n = 100 if full else r.randint(1, min(3 * S + 2, 100)); print(n)
    hi = r.choice([3, 1000])
    a = [r.randint(1, hi) for _ in range(n)]
    if r.random() < 0.2: a.sort(reverse=r.random() < 0.5)
    print(*a)
