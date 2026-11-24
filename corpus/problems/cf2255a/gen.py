import sys, random
if sys.argv[1] == "--max":  # one case n = 1e5 (sum n), k = 1e9, holders at random
    rng = random.Random(0); n = 10**5
    print(1); print(n, 10**9); print("".join(rng.choice("01") for _ in range(2 * n))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); print(n, r.randint(1, 10**5)); print(''.join(r.choice('01') for _ in range(2 * n)))
