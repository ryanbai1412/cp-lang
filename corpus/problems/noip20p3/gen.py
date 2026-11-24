import sys, random
if sys.argv[1] == "--max":  # n = 50, m = 400, colours shuffled at random (no presorted pillars)
    sys.argv[1:] = ["0", "-1"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
# statement: 2 <= n <= 50, 2 <= m <= 400, each colour 1..n appears exactly m times
if S < 0:
    n, m = 50, 400
elif S < 1000:
    n, m = r.randint(2, min(50, 2 + S)), r.randint(2, min(400, 2 + 4 * S))
else:
    n, m = 50 - r.randrange(2), 400 - r.randrange(2)
balls = [c for c in range(1, n + 1) for _ in range(m)]
r.shuffle(balls)
print(n, m)
for i in range(n):
    print(*balls[i * m:(i + 1) * m])
