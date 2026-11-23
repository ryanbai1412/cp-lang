#!/usr/bin/env python3
"""Checker for CF 2089 D: Yes exactly when the reference says Yes (any case),
then an expression of at most 10n + 1000 characters that keeps the string's
digits in order, parses with right-associative ?: and parentheses, and
evaluates to 1."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def parse(e):
    """Value of e, iteratively: a conditional awaits its branches on a stack."""
    pos = 0

    def primary():
        nonlocal pos
        if pos < len(e) and e[pos] in "01":
            pos += 1
            return int(e[pos - 1])
        fail(f"expected a digit or '(' at {pos}")

    stack = []  # frames: ('(',), ('?', cond), (':', cond, then)
    while True:
        while pos < len(e) and e[pos] == "(":
            stack.append(("(",))
            pos += 1
        v = primary()
        while True:
            if pos < len(e) and e[pos] == "?":
                stack.append(("?", v))
                pos += 1
                break
            if stack and stack[-1][0] == "?" and pos < len(e) and e[pos] == ":":
                cond = stack.pop()[1]
                stack.append((":", cond, v))
                pos += 1
                break
            if stack and stack[-1][0] == ":":
                _, cond, then = stack.pop()
                v = then if cond else v
                continue
            if stack and stack[-1][0] == "(" and pos < len(e) and e[pos] == ")":
                stack.pop()
                pos += 1
                continue
            if not stack and pos == len(e):
                return v
            fail(f"unexpected {'end' if pos == len(e) else repr(e[pos])} at {pos}")


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp, out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[1:])
    t, po, pr = int(inp[0]), 0, 0
    for case in range(1, t + 1):
        n, s = int(inp[2 * case - 1]), inp[2 * case]
        want = ref[pr].lower() == "yes"
        pr += 2 if want else 1
        got = out[po].lower()
        po += 1
        if got not in ("yes", "no") or (got == "yes") != want:
            fail(f"case {case}: answered {out[po - 1]}, the answer is {'Yes' if want else 'No'}")
        if not want:
            continue
        e = out[po]
        po += 1
        if len(e) > 10 * n + 1000:
            fail(f"case {case}: expression has {len(e)} characters")
        if "".join(c for c in e if c in "01") != s:
            fail(f"case {case}: digits differ from the string")
        if parse(e) != 1:
            fail(f"case {case}: the expression evaluates to 0")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
