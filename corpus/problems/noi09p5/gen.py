import sys, random
# seed size: pipes of n, m <= size + 1 marbles over A/B (often one letter heavy); --max: n = m = 500
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 499 if mx else int(sys.argv[2])
n = 500 if mx else r.randint(1, min(500, S + 1)); m = 500 if mx else r.randint(1, min(500, S + 1))
pa = r.choice([0.5, 0.9, 1.0])
s = lambda k: "".join("A" if r.random() < pa else "B" for _ in range(k))
print(n, m); print(s(n)); print(s(m))
