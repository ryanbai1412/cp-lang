import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: N x N grid, N <= 1000, cells '.' or A..E; size caps N
n = r.randint(1, max(1, S)) if S <= 50 else S
dens = r.choice([0.05, 0.3, 0.8, 1.0])
letters = r.choice(["ABCDE", "ABCDE", "AB", "E"])
print(n)
for _ in range(n):
    print("".join(r.choice(letters) if r.random() < dens else "." for _ in range(n)))
