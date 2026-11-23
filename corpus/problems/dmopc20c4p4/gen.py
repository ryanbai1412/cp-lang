"""gen.py seed size: DMOPC '20 Contest 4 P4. 1 <= N <= 2e6, 0 <= a_i < i.
size >= 2e6 gives N = 2e6 with shapes by seed: uniform a_i; a_i near i; half zeros then
a_i = i - 1 (every reopen reaches far back, answers near 1e12); a sawtooth."""
import random, sys
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = 2 * 10 ** 6 if size >= 2 * 10 ** 6 else rng.randint(1, size + 1)
shape = seed % 4 if size >= 2 * 10 ** 6 else rng.randrange(4)
a = []
for i in range(1, n + 1):
    if shape == 0: x = rng.randrange(i)
    elif shape == 1: x = max(0, i - 1 - rng.randint(0, 3))
    elif shape == 2: x = 0 if i <= n // 2 else i - 1
    else: x = (i * 7919 % 1000) % i
    a.append(x)
sys.stdout.write(f"{n}\n{' '.join(map(str, a))}\n")
