import sys, random

seed, S = (7, 150) if "--max" in sys.argv[1:] else (int(sys.argv[1]), int(sys.argv[2]))
r = random.Random(seed)
full = S >= 8 and seed % 64 == 7
top = 5000 if full else min(5000, 2 + S * S * 4)
t = r.randint(1, 10_000 if full else 20)
print(t)
for i in range(t):
    if i == 0 and full:
        print(5000, 5000)
    else:
        print(r.randint(1, top), r.randint(1, top))
