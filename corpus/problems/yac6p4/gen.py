import sys, random
if sys.argv[1] == "--max":  # N = 1e6 boards, M = 100, sides spread over all of 1..=100
    r = random.Random(0); n = 10**6
    print(n, 100); print(*(r.randint(1, 100) for _ in range(n))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: 1 <= N <= 1e6, 1 <= M, s_i <= 100; size caps N
n = r.randint(1, max(1, S)) if S <= 1000 else S
cap = r.choice([3, 30, 100])
m = r.randint(1, cap)
if r.random() < 0.3:
    s = [r.randint(1, cap)] * n
else:
    s = [r.randint(1, cap) for _ in range(n)]
print(n, m)
print(*s)
