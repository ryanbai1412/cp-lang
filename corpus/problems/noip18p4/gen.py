import sys, random
# seed size: a random tree on n <= size + 2 towns, plus (half the time) one extra edge making
# a single cycle; no loops or parallel roads. --max: n = 5000 with a cycle
if sys.argv[1] == "--max": sys.argv[1:] = ["0", "-1"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 5000 if S < 0 else r.randint(1, S + 2)
es = {(r.randint(1, v - 1), v) for v in range(2, n + 1)}
if n >= 3 and (S < 0 or r.random() < 0.5):
    while True:
        u, v = sorted(r.sample(range(1, n + 1), 2))
        if (u, v) not in es: es.add((u, v)); break
es = [e if r.random() < 0.5 else e[::-1] for e in es]; r.shuffle(es)
print(n, len(es))
for u, v in es: print(u, v)
