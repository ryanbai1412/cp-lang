#!/usr/bin/env python3
"""Each run-time check of the interpreter, on the smallest program that trips it.

Library tests only prove that correct code passes. This proves the other
half: that wrong code is stopped, and with which message.

    python3 proto/cpi/checks_test.py
"""
import contextlib
import io
import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from cpi.runner import run_tests  # noqa: E402

CASES = [
    ("an index from another domain", """
    domain V = 0..3
    domain E = 0..3
    let a = [V] v => 0
    let x = a[E(2)]
""", "this array is indexed by V, got a E"),
    ("a value outside its domain", """
    domain V = 0..3
    let v = V(3)
""", "3 is not in domain V (0..=2)"),
    ("Int overflow", """
    var x = 1
    for _ in 0..70: x = x * 2
""", "does not fit in 64 bits"),
    ("division by zero", """
    let z = 0
    let b = 5 / z
""", "division by zero"),
    ("arithmetic on none", """
    let a: ?Int = none
    let b = a + 1
""", "no `+` for none and 1"),
    ("ordering none", """
    let a: ?Int = none
    assert a < 1
""", "on `none`: unwrap with `!` or `??` first"),
    ("a list index out of range", """
    let xs = [1, 2, 3]
    let y = xs[3]
""", "index 3 is out of bounds for a list of length 3"),
    ("a failed assert shows both sides", """
    let xs = [1, 2, 3]
    assert xs.len == 4
""", "left:  3"),
    ("a law that does not hold", """
    assert laws_hold(Monoid<Int>(0, (a, b) => a - b), [1, 2, 3])
""", "breaks law `assoc` for a = 1, b = 1, c = 1"),
    ("Euclidean division passes", """
    assert -7 / 2 == -4 and -7 % 2 == 1 and 7 / -2 == -3 and 7 % -2 == 1
""", None),
]


def main():
    bad = 0
    with tempfile.TemporaryDirectory() as d:
        for i, (what, body, want) in enumerate(CASES):
            path = os.path.join(d, f"case{i}.cp")
            with open(path, "w") as f:
                f.write("use std.algebra.{Monoid}\n\n@test\nfn t():" + body)
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                failures = run_tests([path])
            text = out.getvalue()
            ok = (failures == 0) if want is None else (failures == 1 and want in text)
            print(("ok   " if ok else "FAIL ") + what)
            if not ok:
                bad += 1
                print("     wanted: " + (want or "a pass"))
                print("     " + text.strip().replace("\n", "\n     "))
    print(f"{len(CASES) - bad} passed, {bad} failed")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
