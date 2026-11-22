import sys, random

r = random.Random(int(sys.argv[1]) if sys.argv[1] != "--max" else 1)
if sys.argv[1] == "--max":
    tests = [(2, 100000, 3), (100000, 2, 200000), (447, 447, 40)]
else:
    S = int(sys.argv[2])
    tests = []
    for _ in range(r.randint(1, 4)):
        n, m = r.randint(2, 2 + S), r.randint(2, 2 + S // 2)
        tests.append((n, m, r.randint(1, min(n * m, r.choice([1, 2, 4, n * m])))))
print(len(tests))
for n, m, k in tests:
    p = r.choice([0.0, 0.2, 0.5, 0.9])
    print(n, m, k)
    for _ in range(n):
        print(*(-1 if r.random() < p else r.randint(1, k) for _ in range(m)))
