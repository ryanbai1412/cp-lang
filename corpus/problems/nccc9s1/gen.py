import sys, random
# gen.py seed size: T tests of a, b, c with 0 <= a, b, c <= 1e8 and a + b + c > 0.
# No arguments, or size >= 1e8, gives T = 1e4 tests at the full value range.
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = min(int(sys.argv[2]) if len(sys.argv) > 2 else 10 ** 8, 10 ** 8)
r = random.Random(seed)
T = 10 ** 4 if S == 10 ** 8 else r.randint(1, 20)
print(T)
for _ in range(T):
    hi = r.choice([2, 5, S])
    while True:
        a, b, c = (r.choice([r.randint(0, hi), hi, 0, 1, 2]) for _ in range(3))
        if a + b + c > 0: break
    print(a, b, c)
