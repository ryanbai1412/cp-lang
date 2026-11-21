import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = 6
print(t)
for i in range(t):
    if i == 0:
        print(r.randint(1, 15))
    else:
        print(r.randint(1, 15 * S + 5))
