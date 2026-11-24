# IOI 2025 Festival checker (problem-packages/R3): any order of as many coupons as the answer.
# python3 checker.py <input> <output> <answer>; exit 0 accepts. Mirrors the task archive's
# checker/checker.cpp: distinct coupons in 0..N, tokens never negative (capped at 1e15 as
# there), and the count S equal to the reference's.
import sys

CAP = 10**15


def main(inp, out, ans):
    tok = open(inp).read().split()
    n, a = int(tok[0]), int(tok[1])
    price, kind = [int(x) for x in tok[2:2 + 2 * n:2]], [int(x) for x in tok[3:2 + 2 * n:2]]
    want = int(open(ans).read().split()[0])
    got = open(out).read().split()
    try:
        got = [int(x) for x in got]
    except ValueError:
        return "output is not integers"
    if not got:
        return "no output"
    s, buys = got[0], got[1:]
    if s != len(buys):
        return f"S = {s} but {len(buys)} coupons listed"
    if len(set(buys)) != s:
        return "a coupon is bought twice"
    for c in buys:
        if not 0 <= c < n:
            return f"no coupon {c}"
        if a < price[c]:
            return f"coupon {c} costs {price[c]} with {a} tokens left"
        a = min((a - price[c]) * kind[c], CAP)
    if s != want:
        return f"bought {s} coupons, the most is {want}"
    return None


if __name__ == "__main__":
    why = main(*sys.argv[1:4])
    if why:
        print(why, file=sys.stderr)
    sys.exit(1 if why else 0)
