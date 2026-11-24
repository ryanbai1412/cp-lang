import sys, random
# --max: R = C = 3000 holding a shuffled 1..9e6, a 1501 x 1499 window.
if sys.argv[1] == "--max":
    r = random.Random(0)
    p = list(range(1, 9 * 10**6 + 1)); r.shuffle(p)
    print(3000, 3000, 1501, 1499)
    sys.stdout.write("\n".join(" ".join(map(str, p[i * 3000:(i + 1) * 3000])) for i in range(3000)) + "\n")
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
R = r.randint(1, S + 1); C = r.randint(1, S + 1)
h = r.randrange(1, R + 1, 2); w = r.randrange(1, C + 1, 2); print(R, C, h, w)
p = list(range(1, R * C + 1)); r.shuffle(p)
for i in range(R): print(*p[i * C:(i + 1) * C])
