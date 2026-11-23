# ioi05p2 Mountain generator: python3 gen.py SEED [LINES]. Keeps every elevation in [0, 1e9] after
# each reconfiguration (statement), n in 1..=1e9, at most 100000 lines.
import bisect, random, sys

r = random.Random(int(sys.argv[1]))
lines = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([5, 20, 60, 200, 600])
H = 10**9
n = r.choice([1, 2, 3, r.randint(1, 10), r.randint(1, 1000), r.randint(1, 20000), r.randint(1, H), H])

# The track as sorted piece starts (0-based rail cuts) with one slope each.
starts, slopes = [0], [0]


def assign(st, sl, a, b, d):  # rails a..b (0-based half-open) to slope d
    def split(x):
        i = bisect.bisect_right(st, x) - 1
        if st[i] != x:
            st.insert(i + 1, x)
            sl.insert(i + 1, sl[i])
    split(a)
    if b < n:
        split(b)
    i, j = st.index(a), (st.index(b) if b < n else len(st))
    st[i:j], sl[i:j] = [a], [d]


def elevations(st, sl):  # elevation at every piece end; extremes of a linear piece are its ends
    e, out = 0, [0]
    for k in range(len(st)):
        end = st[k + 1] if k + 1 < len(st) else n
        e += sl[k] * (end - st[k])
        out.append(e)
    return out


def rail(lo=0, hi=None):
    hi = n - 1 if hi is None else hi
    cut = r.choice(starts) if r.random() < 0.4 else r.randint(lo, hi)
    return min(max(cut, lo), hi)


out = [str(n)]
for _ in range(lines - 2):
    if r.random() < 0.45:
        for _ in range(30):
            a = rail()
            b = rail(a) + 1 if r.random() < 0.8 else r.randint(a + 1, min(n, a + 3))
            span = b - a
            d = r.choice([0, 1, -1, r.randint(-50, 50), r.randint(-H // span, H // span),
                          H // span, -(H // span), r.randint(-3, 3)])
            st, sl = starts[:], slopes[:]
            assign(st, sl, a, b, d)
            if all(0 <= e <= H for e in elevations(st, sl)):
                starts, slopes = st, sl
                out.append(f"I {a + 1} {b} {d}")
                break
        else:
            out.append(f"Q {r.randint(0, H)}")
    else:
        es = elevations(starts, slopes)
        h = r.choice([0, H, r.randint(0, H), r.choice(es), max(0, r.choice(es) - 1), min(H, r.choice(es) + 1)])
        out.append(f"Q {h}")
out.append("E")
print("\n".join(out))
