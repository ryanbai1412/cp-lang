import sys, random
if sys.argv[1] == "--max":  # sum n = 1e5: one n = 1e5 string over two letters (the most distinct substrings per state)
    rng = random.Random(0); n = 10**5
    print(1); print(n); print("".join(rng.choice("ab") for _ in range(n))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(1, 4 * S * S); k = r.choice([1, 2, 3, 26])
    print(n); print(''.join(chr(97 + r.randrange(k)) for _ in range(n)))
