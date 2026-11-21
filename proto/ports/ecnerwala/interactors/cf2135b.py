#!/usr/bin/env python3
from _lib import run_interactor


def drive(case_text, io):
    toks = list(map(int, case_text.split()))
    it = iter(toks)
    tc = next(it)
    io.send(str(tc))
    for _ in range(tc):
        n = next(it)
        rx0, ry0 = next(it), next(it)
        rx, ry = rx0, ry0
        anchors = [(next(it), next(it)) for _ in range(n)]
        io.send(str(n))
        for ax, ay in anchors:
            io.send(f"{ax} {ay}")
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                d, k = parts[1], int(parts[2])
                if d == "U":
                    ry += k
                elif d == "D":
                    ry -= k
                elif d == "R":
                    rx += k
                elif d == "L":
                    rx -= k
                else:
                    raise RuntimeError(f"bad dir {d}")
                best = min(abs(rx - ax) + abs(ry - ay) for ax, ay in anchors)
                io.send(str(best))
            elif parts[0] == "!":
                gx, gy = int(parts[1]), int(parts[2])
                if gx != rx0 or gy != ry0:
                    raise RuntimeError(f"wrong ({gx}, {gy}) != ({rx0}, {ry0})")
                break
            else:
                raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
