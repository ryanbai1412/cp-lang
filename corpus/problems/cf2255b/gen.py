import sys, random
# `--max`: 5 tests of n = 2e5 (sum n <= 1e6), half 0/1 noise (~n/2 runs) and half long runs.
if sys.argv[1] == "--max":
    r = random.Random(0); print(5)
    for i in range(5):
        n = 200_000; p = 0.5 if i % 2 == 0 else 0.999
        cur, out = '0', []
        for _ in range(n):
            if r.random() > p if p > 0.9 else r.random() < 0.5:
                cur = '1' if cur == '0' else '0'
            out.append(cur)
        print(n); print(''.join(out))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); p = r.choice([0.1, 0.5, 0.9]); print(n); print(''.join('1' if r.random() < p else '0' for _ in range(n)))
