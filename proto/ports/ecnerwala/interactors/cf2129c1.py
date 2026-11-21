#!/usr/bin/env python3
from _lib import run_interactor


def count_balanced_substrings(t):
    st = []
    dp = [0] * len(t)
    tot = 0
    for j, ch in enumerate(t):
        if ch == "(":
            st.append(j)
        else:
            if st:
                p = st.pop()
                dp[j] = 1 + (dp[p - 1] if p > 0 else 0)
                tot += dp[j]
    return tot


def drive(case_text, io):
    lines = [l.strip() for l in case_text.splitlines() if l.strip()]
    tc = int(lines[0])
    io.send(str(tc))
    for i in range(tc):
        s = lines[i + 1]
        n = len(s)
        io.send(str(n))
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                k = int(parts[1])
                idxs = [int(x) for x in parts[2:]]
                if len(idxs) != k:
                    raise RuntimeError(f"bad len {len(idxs)} != {k}")
                t = "".join(s[idx - 1] for idx in idxs)
                io.send(str(count_balanced_substrings(t)))
            elif parts[0] == "!":
                guess = parts[1]
                if guess != s:
                    raise RuntimeError(f"wrong {guess} != {s}")
                break
            else:
                raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
