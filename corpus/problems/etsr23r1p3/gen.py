import sys, random
# Multitest: m letters, a string over them. Small m so some strings hold every ordering.
if sys.argv[1] == "--max":
    r = random.Random(1)
    tests = [(21, 440), (22, 450), (26, 500)]
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    tests = []
    for _ in range(r.randint(1, 4)):
        m = r.randint(1, 4)
        tests.append((m, r.randint(1, max(1, min(S, 12)) + m * m)))
print(len(tests))
for m, n in tests:
    print(m)
    print(''.join(chr(97 + r.randrange(m)) for _ in range(n)))
