#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    secret = int(case_text.split()[0])
    while True:
        parts = io.recv_line().split()
        if parts[0] == "guess" and -50 <= int(parts[1]) <= 50:
            io.send(str(abs(secret - int(parts[1]))))
        elif parts[0] == "answer":
            if int(parts[1]) != secret:
                raise RuntimeError(f"wrong {parts[1]} != {secret}")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
