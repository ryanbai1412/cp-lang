import sys, random
# statement: 1 <= V <= 300 villages at distinct increasing positions in 1..=10000, 1 <= P <= min(30, V).
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "300"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 300
v = 300 if big else r.randint(1, max(1, min(S, 300)))
p = 30 if big else r.randint(1, min(30, v))
span = 10000 if big or r.random() < 0.5 else max(v, 3 * v)
print(v, p)
print(*sorted(r.sample(range(1, span + 1), v)))
