#!/usr/bin/env python3
"""Judge for IOI 2016 P5: "A s" adds s (at most w), "C" compiles, "Q s" asks membership of s
among the compiled elements (at most r), then "! p" must be the hidden permutation."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    tokens = case_text.split()
    n, w, r = int(tokens[0]), int(tokens[1]), int(tokens[2])
    p = [int(x) for x in tokens[3:3 + n]]
    io.send(f"{n} {w} {r}")
    written, asked, compiled = set(), 0, None

    def bits(parts):
        if len(parts) != 2 or len(parts[1]) != n or set(parts[1]) - {"0", "1"}:
            raise RuntimeError(f"bad string {parts}")
        return parts[1]

    while True:
        parts = io.recv_line().split()
        if parts[0] == "A":
            if compiled is not None or len(written) >= w:
                raise RuntimeError("write after compile or over w")
            written.add(bits(parts))
            io.send("1")
        elif parts[0] == "C":
            if compiled is not None:
                raise RuntimeError("compiled twice")
            compiled = {"".join(x[p[i]] for i in range(n)) for x in written}
            io.send("1")
        elif parts[0] == "Q":
            asked += 1
            if compiled is None or asked > r:
                raise RuntimeError("query before compile or over r")
            io.send("1" if bits(parts) in compiled else "0")
        elif parts[0] == "!":
            if [int(x) for x in parts[1:]] != p:
                raise RuntimeError("wrong permutation")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
