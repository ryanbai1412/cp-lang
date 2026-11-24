import sys, random
if sys.argv[1] == "--max":  # t = 100 cases of n = 100 (no sum bound)
    rng = random.Random(0); print(100)
    for _ in range(100):
        s = [rng.choice("01") for _ in range(100)]; s[rng.randrange(100)] = "1"
        print(100); print("".join(s))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, S + 1); s = [r.choice("01") for _ in range(n)]; s[r.randrange(n)] = "1"
    print(n); print("".join(s))
