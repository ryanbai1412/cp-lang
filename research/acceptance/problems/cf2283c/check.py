import sys, importlib.util, os
spec = importlib.util.spec_from_file_location("brute", os.path.join(os.path.dirname(__file__), "brute.py"))
b = importlib.util.module_from_spec(spec); spec.loader.exec_module(b)
out = iter(open(sys.argv[2]).read().split())
try:
    for n in list(map(int, open(sys.argv[1]).read().split()))[1:]:
        d = [int(next(out)) for _ in range(n)]
        if any(x not in (0, 1) for x in d) or not b.ok(n, d): sys.exit(1)
except (StopIteration, ValueError):
    sys.exit(1)
sys.exit(0 if next(out, None) is None else 1)
