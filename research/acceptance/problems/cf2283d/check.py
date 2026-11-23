import sys, importlib.util, os
spec = importlib.util.spec_from_file_location("brute", os.path.join(os.path.dirname(__file__), "brute.py"))
b = importlib.util.module_from_spec(spec); spec.loader.exec_module(b)
out = iter(open(sys.argv[2]).read().split())
try:
    for n, edges in b.cases(open(sys.argv[1]).read()):
        k = int(next(out))
        if not 1 <= k <= 2 * n or b.score(n, edges, k) != b.score(n, edges, b.best(n, edges)): sys.exit(1)
except (StopIteration, ValueError):
    sys.exit(1)
sys.exit(0 if next(out, None) is None else 1)
