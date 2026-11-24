"""gen.py SEED SIZE | --max: CCC 2022 S2. 3g distinct names in g groups of three, x pairs that
must share a group and y that must not, drawn from those names. --max is x = y = g = 1e5."""
import random, string, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
g = 10 ** 5 if MAX else r.randint(1, 1 + 2 * S)
names = set()
while len(names) < 3 * g:
    names.add("".join(r.choice(string.ascii_uppercase) for _ in range(r.randint(1, 10))))
names = list(names); r.shuffle(names)
def pairs(k):
    out = []
    for _ in range(k):
        a, b = r.sample(names, 2) if r.random() < 0.5 else (names[(i := r.randrange(g)) * 3], names[i * 3 + r.randint(1, 2)])
        out.append((a, b))
    return out
for k in ([10 ** 5] * 2 if MAX else [r.randint(0, 2 * S), r.randint(0, 2 * S)]):
    print(k)
    for a, b in pairs(k): print(a, b)
print(g)
for i in range(g): print(*names[3 * i:3 * i + 3])
