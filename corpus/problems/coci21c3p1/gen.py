import sys, random
# statement: 1 <= k <= n <= 50, 1 <= a_i <= 1000.
if sys.argv[1] == "--max":
    # a square-free ternary word: no pattern repeats twice, so every start and length is tried
    tm = [bin(i).count("1") % 2 for i in range(200)]
    zeros = [i for i, b in enumerate(tm) if b == 0]
    word = [zeros[i + 1] - zeros[i] for i in range(50)]
    print(50, 2)
    print(*word)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, max(1, min(S, 50)))
k = r.randint(1, n)
top = r.choice([1, 2, 3, 1000])
a = [r.randint(1, top) for _ in range(n)]
if r.random() < 0.5 and n // k >= 1:
    ln = r.randint(1, n // k)
    pat = [r.randint(1, top) for _ in range(ln)]
    st = r.randint(0, n - ln * k)
    a[st:st + ln * k] = pat * k
print(n, k)
print(*a)
