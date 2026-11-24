import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
# `seed size`: a string of 1..size + 1 letters over a small or full alphabet (size >= 1e6: 1e6).
seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
n = 10 ** 6 if S >= 10 ** 6 else r.randint(1, 4 * S + 1)
abc = "ab" if seed % 3 == 0 else "abc" if seed % 3 == 1 else "abcdefghijklmnopqrstuvwxyz"
print("".join(r.choice(abc) for _ in range(n)))
