import sys, random

if sys.argv[1] == "--max":
    # t = 99 tests (no sum bound): every n from 2 to 100
    print(99)
    for n in range(2, 101):
        print(n)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    print(rng.randint(2, min(100, size * 5 + 2)))
