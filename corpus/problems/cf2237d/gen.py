import sys, random
if sys.argv[1] == "--max":
    # One test at the limit, n = 10^6 (the sum over tests is bounded the same way).
    r = random.Random(0); n = 10**6
    print(1); print(n); print("".join(r.choice("01") for _ in range(n)))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S + 2); print(n)
    print("".join(r.choice("01") for _ in range(n)))
