"""CCO 2021 P2 inputs: `gen.py seed size`. size < 1e5 gives a small base and digit set; size >= 1e5
gives a statement-max input picked by seed % 4: K = 2 with all 5001 digits in -2500..=2500 (the
widest search); K = 1e6 with D = 5001; a random K with D = K digits; or a K whose digits miss
residues (mostly IMPOSSIBLE). Q = 5 numbers, |n| <= 1e18, including +-1e18 and 0."""
import sys, random

# --max: K = 2 with all 5001 digits in -2500..=2500 (the widest search), Q = 5, |n| up to 1e18.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S < 100000:
    k = r.randint(2, 2 + S); m = r.randint(1, min(3 * S, 2500)); d = r.randint(1, min(2 * m + 1, 3 * S))
    digits = r.sample(range(-m, m + 1), d)
    big = 10 ** r.choice([2, 6, 18])
else:
    mode = int(sys.argv[1]) % 4; m = 2500
    if mode == 0:
        k, digits = 2, list(range(-m, m + 1))
    elif mode == 1:
        k, digits = 10**6, r.sample(range(-m, m + 1), 5001)
    elif mode == 2:
        k = r.randint(2, 5001); digits = r.sample(range(-m, m + 1), k)
    else:
        k = r.randint(2, 400); digits = [x for x in range(-m, m + 1) if x % k != 1][:5001]
    big = 10**18
nums = [r.choice([big, -big, 0, r.randint(-big, big)]) for _ in range(5)]
print(k, 5, len(digits), m)
print(*digits)
print("\n".join(map(str, nums)))
