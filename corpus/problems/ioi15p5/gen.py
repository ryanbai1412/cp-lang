# usage: gen.py SEED [MAX_N]: one input in the sample grader's format (N; S; M; M lines X Y), shaped like a random subtask.
import random, sys
if sys.argv[1] == "--max":
    r = random.Random(0)
    n, m = 200000, 600000
    s = list(range(n)); r.shuffle(s)
    print(n); print(*s); print(m)
    print("\n".join(f"{r.randrange(n)} {r.randrange(n)}" for _ in range(m)))
    sys.exit()

r = random.Random(int(sys.argv[1]))
cap = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([5, 30, 100, 500, 2000])
sub = r.choice([1, 2, 3, 4, 5, 6])
if sub == 1:
    n = r.randint(1, min(5, cap)); m = n * n
elif sub in (2, 3):
    n = r.randint(2 if sub == 3 else 1, min(100, cap)); m = 30 * n
elif sub == 4:
    n = r.randint(1, min(500, cap)); m = 30 * n
else:
    n = r.randint(6, max(6, min(200000, cap))); m = 3 * n
s = list(range(n))
shape = r.choice(["random", "random", "sorted", "reversed", "few"])
if shape == "random":
    r.shuffle(s)
elif shape == "reversed":
    s.reverse()
elif shape == "few":
    for _ in range(r.randint(1, 3)):
        i, j = r.randrange(n), r.randrange(n)
        s[i], s[j] = s[j], s[i]
print(n)
print(*s)
print(m)
for _ in range(m):
    if sub in (1, 2):
        print(0, 0)
    elif sub == 3:
        print(0, 1)
    elif r.random() < 0.1:
        i = r.randrange(n); print(i, i)
    else:
        print(r.randrange(n), r.randrange(n))
