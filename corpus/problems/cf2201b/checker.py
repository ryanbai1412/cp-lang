#!/usr/bin/env python3
"""Checker for CF 2201 B: YES exactly when the reference says YES, then cards
1,1,..,n,n on which the statement's greedy player wins in exactly k turns."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def turns(a):
    seen, ready, nxt, done, count = set(), 0, 0, 0, 0
    total = len(a) // 2
    while done < total:
        count += 1
        if ready:
            ready -= 1
        else:
            x = a[nxt]
            nxt += 1
            if x in seen:
                seen.discard(x)
            else:
                y = a[nxt]
                nxt += 1
                if y != x:
                    seen.add(x)
                    if y in seen:
                        ready += 1
                        seen.discard(y)
                    else:
                        seen.add(y)
                    continue
        done += 1
    return count


def main():
    inp, out, ref = load()
    t, po, pr = int(inp[0]), 0, 0
    for case in range(1, t + 1):
        n, k = int(inp[2 * case - 1]), int(inp[2 * case])
        want = ref[pr].upper() == "YES"
        pr += 1 + (2 * n if want else 0)
        got = out[po].upper()
        po += 1
        if got not in ("YES", "NO") or (got == "YES") != want:
            fail(f"case {case}: answered {out[po - 1]}, the answer is {'YES' if want else 'NO'}")
        if not want:
            continue
        a = [int(x) for x in out[po:po + 2 * n]]
        po += 2 * n
        if len(a) != 2 * n or sorted(a) != sorted(list(range(1, n + 1)) * 2):
            fail(f"case {case}: each of 1..{n} must appear exactly twice")
        got_k = turns(a)
        if got_k != k:
            fail(f"case {case}: the greedy takes {got_k} turns, not {k}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
