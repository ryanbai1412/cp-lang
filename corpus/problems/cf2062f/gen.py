import sys, random

if sys.argv[1] == "--max":
    r = random.Random(1)
    tests = [3000]
else:
    r = random.Random(int(sys.argv[1]))
    S = int(sys.argv[2])
    tests = [r.randint(2, max(2, S // 3)) for _ in range(r.randint(1, 4))]
hi = r.choice([5, 1000, 10**9])
print(len(tests))
for n in tests:
    print(n)
    for _ in range(n):
        print(r.randint(0, hi), r.randint(0, hi))
