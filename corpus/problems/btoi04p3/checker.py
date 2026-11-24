"""btoi04p3 checker: `checker.py input output answer`. The least sum matches the answer's, and
the z that follows is strictly increasing, within |z| <= 2e9 and reaches that sum."""
import sys

t = list(map(int, open(sys.argv[1]).read().split()))[1:]
out = open(sys.argv[2]).read().split()
best = int(open(sys.argv[3]).read().split()[0])
if len(out) != len(t) + 1 or int(out[0]) != best:
    sys.exit(1)
z = list(map(int, out[1:]))
if any(abs(x) > 2 * 10**9 for x in z) or any(a >= b for a, b in zip(z, z[1:])):
    sys.exit(1)
sys.exit(0 if sum(abs(a - b) for a, b in zip(t, z)) == best else 1)
