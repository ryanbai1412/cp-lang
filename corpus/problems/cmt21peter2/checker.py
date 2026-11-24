"""Accepts each printed distance sum within 3% of the answer's (the statement allows a few
percent): `checker.py input output answer`."""
import sys
got = open(sys.argv[2]).read().split()
want = open(sys.argv[3]).read().split()
if len(got) != len(want):
    sys.exit(f"{len(got)} numbers, want {len(want)}")
for i, (g, w) in enumerate(zip(got, want)):
    g, w = float(g), float(w)
    if abs(g - w) > 0.03 * max(abs(w), 1.0):
        sys.exit(f"point {i + 1}: {g}, want {w}")
