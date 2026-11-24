import sys, random
# statement: 2 <= M <= 200 regions, 3 <= N <= 250 towns, 1 <= L <= min(30, N) members in distinct
# towns (increasing). Towns sit on a convex polygon cut by non-crossing chords; each inner region is
# listed clockwise from any town, the outer one last, counterclockwise.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "250"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 250
n = 250 if big else r.randint(3, max(3, min(S, 250)))
m = 200 if big else r.randint(2, min(200, n - 1))
faces = [list(range(n))]  # positions clockwise
while len(faces) < m - 1:
    f = r.choice([f for f in faces if len(f) >= 4])
    i = r.randrange(len(f))
    j = (i + r.randint(2, len(f) - 2)) % len(f)
    i, j = min(i, j), max(i, j)
    faces.remove(f)
    faces += [f[i:j + 1], f[j:] + f[:i + 1]]
r.shuffle(faces)
label = list(range(1, n + 1))
r.shuffle(label)
k = 30 if big else r.randint(1, min(30, n))
print(m)
print(n)
print(k)
print(*sorted(r.sample(range(1, n + 1), k)))
for f in faces + [list(range(n))[::-1]]:
    t = r.randrange(len(f))
    f = f[t:] + f[:t]
    print(len(f))
    print(*(label[x] for x in f))
