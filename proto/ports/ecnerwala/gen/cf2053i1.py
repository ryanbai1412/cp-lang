import sys, random

if sys.argv[1] == "--max":
    r = random.Random(1)
    n = 3 * 10**6
    half = [r.randint(1, 10**9) for _ in range((n - 1) // 2)]
    a = half + [-x for x in half]
    r.shuffle(a)
    a.append(10**9)
    print(1)
    print(len(a))
    print(*a)
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 4)
print(t)
for _ in range(t):
    n = r.randint(1, 3 * S)
    hi = r.choice([1, 3, 10, 10**9])
    while True:
        a = [r.randint(-hi, hi) for _ in range(n)]
        if sum(a) >= max(map(abs, a)):
            break
        if sum(a) < 0 and -sum(a) >= max(map(abs, a)):
            a = [-x for x in a]
            break
        n = r.randint(1, 3 * S)
    print(n)
    print(*a)
