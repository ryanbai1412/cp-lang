import sys, random
if sys.argv[1] == "--max":  # sum |s| = 1e6: one string of 1e6 over two letters (many equal-run ties)
    rng = random.Random(0); print(1); print("".join(rng.choice("ab") for _ in range(10**6))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(2, 4 * S + 3)
    alpha = r.choice(["ab", "abc", "abcd"])
    print("".join(r.choice(alpha) for _ in range(n)))
