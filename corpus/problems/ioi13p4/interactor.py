#!/usr/bin/env python3
# Judge for Cave: sends N; `try s_0 .. s_{N-1}` (each 0 or 1) gets the first closed door, or -1
# when all are open, at most 70 000 tries; `answer S_0 .. S_{N-1} D_0 .. D_{N-1}` ends the run.
# The case is N, each switch's open position, each switch's door.
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 70_000


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    n = t[0]
    open_, door = t[1 : 1 + n], t[1 + n : 1 + 2 * n]
    # A try is n single-digit tokens, read as one big integer with a byte per
    # switch; xor-ing the open positions leaves the closed switches.
    # reach[d] has the bytes of the switches wired to doors 0..=d, so the first
    # closed door is the first d whose reach meets them.
    want = int.from_bytes(bytes(48 + o for o in open_), "big")
    switch_of = sorted(range(n), key=door.__getitem__)
    reach, mask = [], 0
    for k in switch_of:
        mask |= 0xFF << 8 * (n - 1 - k)
        reach.append(mask)
    spaces = b" " * n
    src = getattr(io.proc.stdout, "buffer", io.proc.stdout)
    io.send(str(n))
    tries = 0
    while True:
        line = src.readline()
        if not line:
            raise RuntimeError("unexpected EOF from solution")
        line = line.rstrip()
        if line.startswith(b"try") and len(line) == 3 + 2 * n and line[3::2] == spaces \
                and not line[4::2].translate(None, b"01"):
            tries += 1
            if tries > LIMIT:
                raise RuntimeError(f"over {LIMIT} tries")
            closed = int.from_bytes(line[4::2], "big") ^ want
            if closed == 0:
                io.send("-1")
                continue
            lo, hi = 0, n - 1
            while lo < hi:
                mid = (lo + hi) // 2
                if closed & reach[mid]:
                    hi = mid
                else:
                    lo = mid + 1
            io.send(str(lo))
            continue
        parts = line.decode().split()
        if parts[:1] == ["answer"] and len(parts) == 2 * n + 1:
            got = list(map(int, parts[1:]))
            if got != open_ + door:
                raise RuntimeError("wrong answer")
            sys.stderr.write(f"tries {tries}\n")
            return
        raise RuntimeError(f"unexpected {parts[:3]}")


if __name__ == "__main__":
    run_interactor(drive)
