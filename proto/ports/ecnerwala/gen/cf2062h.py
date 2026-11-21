import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S >= 1000:
    sizes = [14]
else:
    sizes = []
    budget = 1 << min(14, S + 1)
    while True:
        n = r.randint(1, min(14, S + 1))
        if (1 << n) > budget:
            break
        budget -= 1 << n
        sizes.append(n)
        if len(sizes) >= 6 or r.random() < 0.3:
            break
    if not sizes:
        sizes = [1]
print(len(sizes))
for n in sizes:
    p = r.choice([0.1, 0.3, 0.5, 0.8, 1.0])
    print(n)
    for _ in range(n):
        print("".join("1" if r.random() < p else "0" for _ in range(n)))
