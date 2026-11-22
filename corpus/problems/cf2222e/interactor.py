#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    toks = list(map(int, case_text.split()))
    it = iter(toks)
    t = next(it)
    io.send(str(t))
    for _ in range(t):
        n = next(it)
        k_ans = next(it)
        c_ans = next(it)
        io.send(str(n))
        init = int(io.recv_line())
        st = {init}
        ops = 0
        while True:
            parts = io.recv_line().split()
            cmd = parts[0]
            if cmd == "I":
                ops += 1
                v = int(parts[1])
                if k_ans == 1:
                    st.add(v & c_ans)
                elif k_ans == 2:
                    st.add(v | c_ans)
                elif k_ans == 3:
                    st.add(v ^ c_ans)
                else:
                    raise RuntimeError(f"bad k_ans {k_ans}")
                io.send(str(len(st)))
            elif cmd == "Q":
                ops += 1
                v = int(parts[1])
                io.send(str(sum(1 for z in st if z >= v)))
            elif cmd == "A":
                kg, cg = int(parts[1]), int(parts[2])
                if kg != k_ans or cg != c_ans:
                    raise RuntimeError(f"wrong answer ({kg}, {cg}) != ({k_ans}, {c_ans})")
                if ops > n + 3:
                    raise RuntimeError(f"too many ops: {ops} > {n + 3}")
                break
            else:
                raise RuntimeError(f"unexpected cmd: {parts}")


if __name__ == "__main__":
    run_interactor(drive)
