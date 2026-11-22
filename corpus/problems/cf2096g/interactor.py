#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    toks = iter(case_text.split())
    t = int(next(toks))
    io.send(str(t))
    for _ in range(t):
        n = int(next(toks))
        fake = int(next(toks))  # 1-indexed
        hide = int(next(toks))  # -1 or 0-indexed query to mask with '?'
        io.send(str(n))
        q = int(io.recv_line().strip())
        if q < 1 or q > 20:
            raise RuntimeError(f"bad q={q} for n={n}")
        resp = []
        for qi in range(q):
            nums = list(map(int, io.recv_line().split()))
            k = nums[0]
            if k % 2 != 0 or len(nums) != k + 1:
                raise RuntimeError(f"bad query line {nums}")
            half = k // 2
            left = set(nums[1:1 + half])
            right = set(nums[1 + half:])
            if len(left) != half or len(right) != half or (left & right):
                raise RuntimeError(f"duplicate elements in query {nums}")
            if qi == hide:
                resp.append("?")
            elif fake in left:
                resp.append("L")
            elif fake in right:
                resp.append("R")
            else:
                resp.append("N")
        io.send("".join(resp))
        ans = int(io.recv_line().strip())
        if ans != fake:
            raise RuntimeError(f"wrong answer {ans}, expected {fake} (n={n}, hide={hide})")


run_interactor(drive)
