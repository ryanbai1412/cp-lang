import sys, random
# statement: 2 <= N <= 1000; general i (1..N-1) ranks 0..i-1, a permutation.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 1000 if S >= 1000 else r.randint(2, max(2, min(S, 1000)))
mode = r.randint(0, 2)
print(n)
for i in range(1, n):
    rank = list(range(i))
    if mode == 0:
        r.shuffle(rank)
    elif mode == 1:
        rank.reverse()
    print(*rank)
