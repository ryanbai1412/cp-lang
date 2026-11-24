import sys, random
if len(sys.argv) > 1 and sys.argv[1] == "--max":  # sum n = 1e6: one n = 1e6 test
    sys.argv[1:] = ["0", str(10**6)]
# gen.py seed size: t tests of n <= size + 1 values in 1..=1e6, sum n <= 1e6.
# No arguments, or size >= 1e6: seed even gives one test with n = 1e6, seed odd gives
# t = 1e4 tests of n = 100.
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = int(sys.argv[2]) if len(sys.argv) > 2 else 10 ** 6
r = random.Random(seed)
if S >= 10 ** 6:
    ns = [10 ** 6] if seed % 2 == 0 else [100] * 10 ** 4
else:
    ns = [r.randint(1, min(S + 1, 6)) if r.random() < 0.5 else r.randint(1, S + 1) for _ in range(5)]
print(len(ns))
for n in ns:
    print(n); print(*[r.randint(1, r.choice([3, 100, 10 ** 6])) for _ in range(n)])
