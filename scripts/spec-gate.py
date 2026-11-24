#!/usr/bin/env python3
"""The one entry for every spec and conformance gate: `spec-gate.py <gate> [args]`.

`spec-gate.py --list` names them. Each gate lives in `scripts/specgate/`
and has a test in `scripts/test_spec_gate.py`. The gap gates enforce
`handbook/guidelines/conformance-checklist.md#conformance-r3-a-known-gap-fails-for-the-reason-it-names`
(conformance/R3).
"""
from __future__ import annotations

import argparse
import importlib
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

GATES = ["changelog", "decisions-explained", "flakes", "markers", "matrix", "ownership", "prelude", "speed", "std-signatures", "subset", "tool-rules", "traits", "vocab", "widths", "xref"]


def load(name: str):
    return importlib.import_module(f"specgate.{name.replace('-', '_')}")


def main(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help", "--list"):
        for g in GATES:
            print(f"{g:24} {load(g).HELP}")
        return 0 if argv else 2
    name, rest = argv[0], argv[1:]
    if name not in GATES:
        print(f"spec-gate: no gate `{name}`; `--list` names them", file=sys.stderr)
        return 2
    gate = load(name)
    p = argparse.ArgumentParser(prog=f"spec-gate.py {name}", description=gate.HELP)
    gate.args(p)
    problems = gate.run(p.parse_args(rest))
    for pr in problems:
        print(f"spec-gate {name}: {pr.where}: {pr.what}")
    print(f"spec-gate {name}: {len(problems)} problem{'s' * (len(problems) != 1)}" if problems else f"spec-gate {name}: ok")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
