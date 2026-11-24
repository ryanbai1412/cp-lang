"""checker.py input output answer: every number within 1e-6 of the answer's."""
import sys
out = open(sys.argv[2]).read().split()
ans = open(sys.argv[3]).read().split()
if len(out) != len(ans):
    sys.exit(f"wrong count: {len(out)} vs {len(ans)}")
for i, (o, a) in enumerate(zip(out, ans)):
    if abs(float(o) - float(a)) > 1e-6 + 1e-9:
        sys.exit(f"line {i + 1}: {o} vs {a}")
