import sys, random
# seed size: n <= size + 1 doors, t below 2^b for a random b <= 30, m random; --max: n = 1e5, m = 1e9
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 10**5 if mx else int(sys.argv[2])
n = 10**5 if mx else r.randint(1, min(10**5, S + 1))
b = 30 if mx else r.randint(1, 30)
m = 10**9 if mx else r.randint(0, min(10**9, 2 ** b))
print(n, m)
for _ in range(n): print(r.choice(["AND", "OR", "XOR"]), r.randint(0, min(10**9, 2 ** b - 1)))
