import sys, random

if sys.argv[1] == "--max":
    # N = 5000 random over all 62 letters and digits: almost no matching ends
    rng = random.Random(1)
    alpha = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    print(5000); print("".join(rng.choice(alpha) for _ in range(5000)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(3, S + 4); print(n)
print(''.join(r.choice('abC1'[:r.randint(1, 4)]) for _ in range(n)))
