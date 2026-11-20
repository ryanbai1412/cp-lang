import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(2, 4 * S + 3)
    alpha = r.choice(["ab", "abc", "abcd"])
    print("".join(r.choice(alpha) for _ in range(n)))
