#!/usr/bin/env python3
# Judge for YAC 1 P3: sends "N T"; per game reads 1 or 2 (who moves first), then alternates
# moves "r c"; a move must take an unmarked cell with no marked edge-neighbour. The judge
# plays uniformly random legal cells and sends "0 0" when it has none. The case is "N T SEED".
import os, sys, random
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    n, t, seed = map(int, case_text.split())
    r = random.Random(seed)
    io.send(f"{n} {t}")
    for _ in range(t):
        # blocked: marked cells and their edge-neighbours, the cells no one may take.
        blocked = set()
        def take(x, y):
            blocked.update(((x, y), (x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
        def theirs():
            x, y = map(int, io.recv_line().split())
            if not (1 <= x <= n and 1 <= y <= n) or (x, y) in blocked:
                raise RuntimeError(f"illegal move {x} {y}")
            take(x, y)
        cells = [(x, y) for x in range(1, n + 1) for y in range(1, n + 1)]
        r.shuffle(cells)
        first = int(io.recv_line())
        if first not in (1, 2):
            raise RuntimeError(f"bad choice {first}")
        if first == 1:
            theirs()
        while True:
            while cells and cells[-1] in blocked:
                cells.pop()
            if not cells:
                io.send("0 0")
                break
            x, y = cells.pop()
            take(x, y)
            io.send(f"{x} {y}")
            theirs()


if __name__ == "__main__":
    run_interactor(drive)
