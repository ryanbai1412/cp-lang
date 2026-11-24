#!/usr/bin/env python3
"""Checker for NOIP 2022 Meow Meow: each test's plan has m to 2m moves, deals every card in
order with `1 s` (two equal cards on top cancel), cancels equal bottoms with `2 s t`, and
leaves every stack empty."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = iter(map(int, open(sys.argv[1]).read().split()))
    try:
        out = iter(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output is not integers")

    def take():
        try:
            return next(out)
        except StopIteration:
            fail("output ends early")

    for case in range(next(inp)):
        n, m, _ = next(inp), next(inp), next(inp)
        deck = [next(inp) for _ in range(m)]
        ops = take()
        if not m <= ops <= 2 * m:
            fail(f"test {case + 1}: {ops} moves, not in {m}..={2 * m}")
        stacks = [[] for _ in range(n + 1)]
        dealt = 0
        for _ in range(ops):
            kind = take()
            if kind == 1:
                s = take()
                if not 1 <= s <= n:
                    fail(f"test {case + 1}: stack {s} out of range")
                if dealt == m:
                    fail(f"test {case + 1}: deals past the deck")
                card = deck[dealt]; dealt += 1
                if stacks[s] and stacks[s][-1] == card:
                    stacks[s].pop()
                else:
                    stacks[s].append(card)
            elif kind == 2:
                s, t = take(), take()
                if not (1 <= s <= n and 1 <= t <= n) or s == t:
                    fail(f"test {case + 1}: bad stacks {s} {t}")
                if not stacks[s] or not stacks[t] or stacks[s][0] != stacks[t][0]:
                    fail(f"test {case + 1}: bottoms of {s} and {t} differ")
                stacks[s].pop(0); stacks[t].pop(0)
            else:
                fail(f"test {case + 1}: move kind {kind}")
        if dealt != m or any(stacks):
            fail(f"test {case + 1}: deck or stacks not cleared")
    if next(out, None) is not None:
        fail("extra output")


if __name__ == "__main__":
    main()
