import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 5)
print(t)
for _ in range(t):
    m = r.randint(1, 3 * S)
    n = 2 * m
    p_q = r.choice([0.2, 0.5, 0.8, 1.0])
    s = []
    for _ in range(n):
        if r.random() < p_q:
            s.append("?")
        else:
            s.append(r.choice(["a", "b"]))
    print(n)
    print("".join(s))
