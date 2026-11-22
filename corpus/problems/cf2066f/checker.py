# Codeforces 2066 F checker: the same -1 verdicts as the original, and every
# printed sequence of operations turns a into b, each replacing a maximum-sum
# subarray by a non-empty array, with at most n + m replacement values in all.
import sys


def mss(xs):
    best, run = None, 0
    for v in xs:
        run = max(run, 0) + v
        best = run if best is None else max(best, run)
    return best


def main():
    inp = open(sys.argv[1]).read().split()
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    pi = po = pr = 0

    def take(toks, i):
        return int(toks[i]), i + 1

    t, pi = take(inp, pi)
    for case in range(t):
        n, pi = take(inp, pi)
        m, pi = take(inp, pi)
        a = [int(x) for x in inp[pi:pi + n]]
        pi += n
        b = [int(x) for x in inp[pi:pi + m]]
        pi += m
        q, po = take(out, po)
        rq, pr = take(ref, pr)
        if rq != -1:
            for _ in range(rq):
                pr += 2
                k, pr = take(ref, pr)
                pr += k
        assert (q == -1) == (rq == -1), f"case {case}: -1 verdict differs"
        if q == -1:
            continue
        assert 0 <= q, f"case {case}: bad q"
        cur, used = a[:], 0
        for _ in range(q):
            l, po = take(out, po)
            r, po = take(out, po)
            k, po = take(out, po)
            c = [int(x) for x in out[po:po + k]]
            po += k
            assert 1 <= l <= r <= len(cur), f"case {case}: bad range {l} {r}"
            assert k >= 1 and len(c) == k, f"case {case}: empty replacement"
            assert all(abs(v) <= 10**9 for v in c), f"case {case}: value too large"
            assert sum(cur[l - 1:r]) == mss(cur), f"case {case}: {l}..{r} is not a maximum-sum subarray"
            used += k
            cur = cur[:l - 1] + c + cur[r:]
        assert used <= n + m, f"case {case}: {used} replacement values > n + m"
        assert cur == b, f"case {case}: result is not b"


main()
