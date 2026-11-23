"""NOI 2023 P2 inputs: `gen.py seed size`. size < 1e5 gives up to 15 small tests; size >= 1e5 gives a
statement-max input: 15 tests at n = 3e4, m = 3000, k = 10 (seed % 3 == 0), or k mixed over 0..=10,
or n = 1 tests."""
import sys, random

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
tests = []
if S < 100000:
    for _ in range(r.randint(1, 15)):
        tests.append((r.randint(1, 4 * S), r.randint(0, 4 * S), r.randint(0, 10)))
else:
    mode = int(sys.argv[1]) % 3
    for _ in range(15):
        if mode == 0: tests.append((30000, 3000, 10))
        elif mode == 1: tests.append((30000, 3000, r.randint(0, 10)))
        else: tests.append((1, 3000, r.choice([0, 10])))
print(r.randint(1, 20), len(tests))
for n, m, k in tests:
    print(n, m, k)
    print(*(r.randint(1, i - 1) for i in range(2, n + 1)))
