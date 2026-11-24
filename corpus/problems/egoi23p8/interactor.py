#!/usr/bin/env python3
# interactor: spawn
"""Run-twice judge for EGOI 2023 "Guessing Game": `interactor.py CASE -- SOLUTION...`.

CASE is `N SEED`. Emma's house e, the order of the other N - 1 houses and Emma's label come from
SEED. Run 1 gets `1 N` and the N - 1 houses, and prints K <= 1e5 then a label in 1..=K per house.
Emma labels e (1, a middle label, or K, by SEED). Run 2 gets `2 N` and all N labels, and must
print two houses, one of them e."""
import random
import subprocess
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def run(cmd, text):
    p = subprocess.run(cmd, input=text, capture_output=True, text=True, timeout=20)
    if p.returncode != 0:
        fail(f"solution exited {p.returncode}: {p.stderr[-500:]}")
    return p.stdout.split()


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        sys.stderr.write("usage: interactor.py CASE -- SOLUTION...\n")
        sys.exit(2)
    n, seed = map(int, open(sys.argv[1], encoding="utf-8").read().split())
    cmd = sys.argv[3:]
    rng = random.Random(seed)
    e = rng.randrange(n)
    order = [h for h in range(n) if h != e]
    rng.shuffle(order)
    out = run(cmd, f"1 {n}\n" + "\n".join(map(str, order)) + "\n")
    if len(out) != n or not all(w.isdigit() for w in out):
        fail(f"run 1: expected K and {n - 1} labels, got {len(out)} tokens")
    k, labels = int(out[0]), list(map(int, out[1:]))
    if not 1 <= k <= 100000 or not all(1 <= x <= k for x in labels):
        fail(f"run 1: K = {k} or a label is out of 1..=K")
    label = [0] * n
    for h, x in zip(order, labels):
        label[h] = x
    label[e] = [1, k, rng.randint(1, k)][seed % 3] if seed % 7 else rng.randint(1, k)
    got = run(cmd, f"2 {n}\n" + " ".join(map(str, label)) + "\n")
    if len(got) != 2 or str(e) not in got:
        fail(f"run 2: guessed {' '.join(got)[:40]}, Emma's house is {e}")
    sys.stderr.write(f"ok: K = {k}\n")


if __name__ == "__main__":
    main()
