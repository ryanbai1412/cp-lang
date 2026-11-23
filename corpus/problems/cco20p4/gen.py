import sys, random
# seed size: n = size buildings (or fewer), roads red with a random bias, or by a hidden
# split (red inside two groups, blue across) to stress the junction moves
rng = random.Random(int(sys.argv[1])); S = max(2, min(int(sys.argv[2]), 2000))
n = S if S >= 2000 or rng.random() < 0.4 else rng.randint(2, S)
kind = rng.randrange(3)
p = rng.choice([0.0, 0.05, 0.5, 0.95, 1.0, rng.random()])
side = [rng.randrange(2) for _ in range(n)]
print(n)
for i in range(1, n):
    if kind == 2: row = ''.join('R' if side[i] == side[j] else 'B' for j in range(i))
    else: row = ''.join('R' if rng.random() < p else 'B' for j in range(i))
    print(row)
