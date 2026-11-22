#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def parse_expr(tokens):
    pos = 0

    def parse():
        nonlocal pos
        tok = tokens[pos]
        if tok in ("min", "max"):
            pos += 1
            assert tokens[pos] == "("
            pos += 1
            args = [parse()]
            while tokens[pos] == ",":
                pos += 1
                args.append(parse())
            assert tokens[pos] == ")"
            pos += 1
            return (tok, args)
        pos += 1
        return ("var", int(tok) - 1)

    node = parse()
    assert pos == len(tokens), f"extra tokens: {tokens[pos:]}"
    return node


def eval_expr(node, vals):
    kind, data = node
    if kind == "var":
        return vals[data]
    if kind == "min":
        return min(eval_expr(ch, vals) for ch in data)
    return max(eval_expr(ch, vals) for ch in data)


def drive(case_text, io):
    lines = [ln.strip() for ln in case_text.splitlines() if ln.strip()]
    idx = 0
    t = int(lines[idx])
    idx += 1
    io.send(str(t))
    total_q = 0
    for _ in range(t):
        n = int(lines[idx])
        idx += 1
        tree = parse_expr(lines[idx].split())
        idx += 1
        num_evals = int(lines[idx])
        idx += 1
        evals = []
        for _ in range(num_evals):
            evals.append(list(map(int, lines[idx].split())))
            idx += 1

        io.send(str(n))
        while True:
            parts = io.recv_line().split()
            if not parts:
                raise RuntimeError("empty response")
            if parts[0] == "?":
                total_q += 1
                if total_q > 1500:
                    raise RuntimeError(f"too many queries: {total_q} > 1500")
                if len(parts) != n + 1:
                    raise RuntimeError(f"bad query length: {len(parts) - 1} != {n}")
                xs = [int(v) for v in parts[1:]]
                if any(v < 1 or v > 10**9 for v in xs):
                    raise RuntimeError("query value out of [1, 10^9]")
                io.send(str(eval_expr(tree, xs)))
            elif parts[0] == "!":
                break
            else:
                raise RuntimeError(f"unexpected command: {parts[0]}")

        for xs in evals:
            io.send(" ".join(map(str, xs)))
            got = int(io.recv_line())
            want = eval_expr(tree, xs)
            if got != want:
                raise RuntimeError(f"wrong eval: got {got}, want {want}")
        io.send("0")


if __name__ == "__main__":
    run_interactor(drive)
