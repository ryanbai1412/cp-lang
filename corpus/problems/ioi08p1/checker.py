#!/usr/bin/env python3
"""Checker for IOI 2008 Type Printer: the operation count must equal the reference's, and
replaying the operations ('P' prints, '-' deletes the last letter, else types it) must print
every word, never delete from an empty printer and print nothing that is not a word."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = open(sys.argv[1]).read().split()
    words = set(inp[1:1 + int(inp[0])])
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if not out or out[0] != ref[0]:
        fail(f"got {out[:1]} operations, want {ref[0]}")
    ops = out[1:]
    if len(ops) != int(out[0]):
        fail(f"says {out[0]} operations but lists {len(ops)}")
    typed, printed = [], set()
    for op in ops:
        if op == "P":
            word = "".join(typed)
            if word not in words:
                fail(f"prints {word!r}, which is not a word")
            printed.add(word)
        elif op == "-":
            if not typed:
                fail("deletes from an empty printer")
            typed.pop()
        elif len(op) == 1 and "a" <= op <= "z":
            typed.append(op)
        else:
            fail(f"unknown operation {op!r}")
    if printed != words:
        fail(f"{len(words - printed)} words never printed")


main()
