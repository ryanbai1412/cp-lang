import sys, random
# CCC 2002 S4: m (1..=20) max group size, n (1..=100) people, each a name and a time (1..=1000).
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2]) if len(sys.argv) > 2 else 100
n = r.randint(1, min(S, 100)); m = r.randint(1, 20)
hi = r.choice([3, 10, 1000])
print(m); print(n)
for i in range(n): print(f"p{i:03d}"); print(r.randint(1, hi))
