"""SAC '22 CC4 P4 inputs: `gen.py seed size`. size < 1e5 gives n <= 4 size, small values; size >= 1e5
gives n = 1e6, |a| <= 1e9 picked by seed % 4: random values with random B <= T; B = 0, T = 2e9
(every subarray, answer n(n + 1) / 2); alternating +-1e9 with B = T = 2e9; or a slow random walk."""
import sys, random

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2]); E = 10**9
if S < 100000:
    n = r.randint(1, 4 * S); v = r.choice([3, 10, E]); a = [r.randint(-v, v) for _ in range(n)]
    B = r.randint(0, 2 * v); T = r.randint(B, 2 * v)
else:
    n = 10**6; mode = int(sys.argv[1]) % 4
    if mode == 0:
        a = [r.randint(-E, E) for _ in range(n)]; B = r.randint(0, 2 * E); T = r.randint(B, 2 * E)
    elif mode == 1:
        a = [r.randint(-E, E) for _ in range(n)]; B, T = 0, 2 * E
    elif mode == 2:
        a = [E if i % 2 else -E for i in range(n)]; B = T = 2 * E
    else:
        a, x = [], 0
        for _ in range(n):
            x = max(-E, min(E, x + r.randint(-3000, 3000))); a.append(x)
        B = r.randint(0, 10**5); T = r.randint(B, 10**6)
print(n, B, T)
print(*a)
