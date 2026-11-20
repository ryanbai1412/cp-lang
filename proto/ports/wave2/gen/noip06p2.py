import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(60, S + 1)); print(r.randint(1, 3200) * 10, n)
mains = [i for i in range(1, n + 1) if i == 1 or r.random() < .4]; left = {i: 2 for i in mains}
for i in range(1, n + 1):
    p = 0
    if i not in left:
        c = [j for j in mains if left[j]]
        if c: p = r.choice(c); left[p] -= 1
        else: left[i] = 0
    print(r.randint(1, 1000) * 10, r.randint(1, 5), p)
