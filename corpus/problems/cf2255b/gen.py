import sys, random
# `seed size`: 6 tests of n <= 4 size. size >= 1e6 (or `--max`) fills sum n = 1e6, t <= 1e4:
# one n = 1e6 string (noise, alternating 0101..., long runs, all one digit) or 1e4 tests of 100.
def bits(r, n, p):
    return ''.join('1' if r.random() < p else '0' for _ in range(n))
if sys.argv[1] == "--max" or int(sys.argv[2]) >= 10 ** 6:
    seed = 0 if sys.argv[1] == "--max" else int(sys.argv[1])
    r = random.Random(seed); N = 10 ** 6
    if seed % 5 == 4:
        print(10 ** 4)
        for _ in range(10 ** 4): print(100); print(bits(r, 100, 0.5))
        sys.exit()
    print(1); print(N)
    if seed % 5 == 0: print(bits(r, N, 0.5))
    elif seed % 5 == 1: print('01' * (N // 2))
    elif seed % 5 == 2:
        cur, out = '0', []
        for _ in range(N):
            if r.random() < 0.001: cur = '1' if cur == '0' else '0'
            out.append(cur)
        print(''.join(out))
    else: print('1' * N)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); p = r.choice([0.1, 0.5, 0.9]); print(n); print(bits(r, n, p))
