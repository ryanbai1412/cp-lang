# "second" inputs must be what the judge would send: an encoding with one node flipped.
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])

def untwist(h, s):
    size = (1 << h) - 1; out = [None] * size; frm = [1] * (size + 2)
    for i in range(1, size + 1):
        out[i - 1] = s[frm[i] - 1]
        if 2 * i <= size:
            f = out[i - 1]; frm[2 * i] = 2 * frm[i] + f; frm[2 * i + 1] = 2 * frm[i] + 1 - f
    return out

def encode(h, b):
    size = (1 << h) - 1; s = [0] * size; it = iter(b)
    for i in range(size - 1, -1, -1):
        if i == 0 or i & (i - 1): s[i] = next(it, 0)
        else: s[i] = sum(s[j] for j in range(i + 1, size) if j & i) % 2
    return untwist(h, s)

def flip(h, s, p):
    s[p - 1] ^= 1; a, b, l = 2 * p, 2 * p + 1, 1
    while a < (1 << h):
        for k in range(l): s[a + k - 1], s[b + k - 1] = s[b + k - 1], s[a + k - 1]
        a, b, l = 2 * a, 2 * b, 2 * l
    return s

op = r.choice(["first", "second"]); t = 5; print(op); print(t)
for _ in range(t):
    h = r.randint(2, 2 + S); n = r.randint(1, (1 << h) - 1 - h)
    b = [r.randint(0, 1) for _ in range(n)]
    out = b if op == "first" else flip(h, encode(h, b), r.randint(1, (1 << h) - 1))
    print(n, h); print("".join(map(str, out)))
