import sys, random
# seed size: n <= size + 1, flows below a random cap <= 1e6 (often 0 or tiny, to make ties);
# --max: n = 500, flows up to 1e6
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 499 if mx else int(sys.argv[2])
n = 500 if mx else r.randint(1, min(500, S + 1))
cap = 10**6 if mx else r.choice([1, 5, 100, 10**6])
def rows(a, b):
    for _ in range(a): print(*(r.randint(0, cap) for _ in range(b)))
print(n)
rows(n + 1, n); rows(n, n + 1); rows(n + 1, n); rows(n, n + 1)
