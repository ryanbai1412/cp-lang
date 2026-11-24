"""noi08p6 checker: `checker.py input output answer`. The output puts a permutation of 1..n
in the slots with little Z (1) in slot 1, and his expected prize is at least the answer's
(the task is scored against the organizers' bracket, so a better one also passes)."""
import sys

d = open(sys.argv[1]).read().split()
n = int(d[1])
P = [[float(x) for x in d[2 + i * n:2 + (i + 1) * n]] for i in range(n)]
k = n.bit_length() - 1
a = [int(x) for x in d[2 + n * n:3 + n * n + k]]


def prize(path):
    s = open(path).read().split()
    if len(s) != n or not all(x.isdigit() for x in s):
        return None
    s = [int(x) - 1 for x in s]
    if sorted(s) != list(range(n)) or s[0] != 0:
        return None
    alive, e = 1.0, 0.0
    for r in range(k):
        blk = s[1 << r:2 << r]
        w = [1.0] * len(blk)
        b = 1
        while b < len(blk):
            for st in range(0, len(blk), 2 * b):
                L, R = blk[st:st + b], blk[st + b:st + 2 * b]
                wl, wr = w[st:st + b], w[st + b:st + 2 * b]
                for i in range(b):
                    w[st + i] = wl[i] * sum(wr[j] * P[L[i]][R[j]] for j in range(b))
                    w[st + b + i] = wr[i] * sum(wl[j] * P[R[i]][L[j]] for j in range(b))
            b *= 2
        beat = sum(wj * P[0][x] for wj, x in zip(w, blk))
        e += alive * (1 - beat) * a[r]
        alive *= beat
    return e + alive * a[k]


mine, ref = prize(sys.argv[2]), prize(sys.argv[3])
sys.exit(0 if mine is not None and ref is not None and mine >= ref - 1e-9 * max(1.0, abs(ref)) else 1)
