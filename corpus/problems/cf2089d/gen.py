import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    half = rng.randint(1, size * 3 + 2)
    print(half)
    print("".join(rng.choice("01") for _ in range(2 * half + 1)))
