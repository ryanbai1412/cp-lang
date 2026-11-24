#!/usr/bin/env python3
# Judge for IOI 2010 Cluedo, stdio-adapted: the program reads T; for each case it prints
# theories "M L W" and reads 0 (correct, the case is over) or the number of one wrong part
# (1 murderer, 2 location, 3 weapon), chosen at random among the wrong ones. Each case
# allows at most 20 theories (subtask 2). The case is T then T lines "M L W" (gen.py).
import os, random, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 20


def drive(case_text, io):
    head, *rows = case_text.split("\n")
    rng = random.Random(case_text)
    answers = [tuple(map(int, r.split())) for r in rows if r.strip()]
    if len(answers) != int(head):
        raise RuntimeError("bad case: T rows expected")
    io.send(head)
    for k, want in enumerate(answers, 1):
        for _ in range(LIMIT):
            got = tuple(map(int, io.recv_line().split()))
            if len(got) != 3 or not (1 <= got[0] <= 6 and 1 <= got[1] <= 10 and 1 <= got[2] <= 6):
                raise RuntimeError(f"case {k}: bad theory {got}")
            wrong = [i + 1 for i in range(3) if got[i] != want[i]]
            io.send(str(rng.choice(wrong)) if wrong else "0")
            if not wrong:
                break
        else:
            raise RuntimeError(f"case {k}: more than {LIMIT} theories")


if __name__ == "__main__":
    run_interactor(drive)
