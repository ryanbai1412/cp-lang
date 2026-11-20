import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    h = r.randint(1, 3 * S); s = []; o = 0; c = 0
    p = r.choice([0.3, 0.5, 0.7])
    while c < h:
        if o < h and (o == c or r.random() < p): s.append("("); o += 1
        else: s.append(")"); c += 1
    print(2 * h, r.randint(0, h)); print("".join(s))
