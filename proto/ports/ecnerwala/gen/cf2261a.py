import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, S + 1); s = [r.choice("01") for _ in range(n)]; s[r.randrange(n)] = "1"
    print(n); print("".join(s))
