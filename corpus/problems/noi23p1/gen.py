import sys, random
# seed size: n, m <= size (capped at 1e9), q <= 1e5; at most 5 diagonal strokes
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S > 1000
n = min(S, 10**9) if big and r.random() < 0.7 else r.randint(1, min(S, 10**9))
m = min(S, 10**9) if big and r.random() < 0.7 else r.randint(1, min(S, 10**9))
q = 10**5 if big else r.randint(1, max(1, S))
mode = r.randint(0, 2)  # 0 random, 1 few lines (heavy merging), 2 full-length strokes
diag = r.randint(0, min(5, q))
kinds = [3] * diag + [r.randint(1, 2) for _ in range(q - diag)]
r.shuffle(kinds)
xs = [r.randint(1, n) for _ in range(30)]
ys = [r.randint(1, m) for _ in range(30)]
def px(): return r.choice(xs) if mode == 1 else r.randint(1, n)
def py(): return r.choice(ys) if mode == 1 else r.randint(1, m)
print(0)
print(n, m, q)
for t in kinds:
    if t == 1:
        y = py()
        a, b = (1, n) if mode == 2 else sorted((px(), px()))
        print(1, a, y, b, y)
    elif t == 2:
        x = px()
        a, b = (1, m) if mode == 2 else sorted((py(), py()))
        print(2, x, a, x, b)
    else:
        x1, y1 = px(), py()
        k = min(n - x1, m - y1)
        L = k if mode == 2 else r.randint(0, k)
        print(3, x1, y1, x1 + L, y1 + L)
