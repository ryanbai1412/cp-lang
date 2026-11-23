import sys, importlib.util, os
spec = importlib.util.spec_from_file_location("brute", os.path.join(os.path.dirname(__file__), "brute.py"))
b = importlib.util.module_from_spec(spec); spec.loader.exec_module(b)
out = iter(open(sys.argv[2]).read().split())
try:
    for k, ids, edges, nv in b.cases(open(sys.argv[1]).read()):
        add = []
        for _ in range(k - 1):
            t1, v1, t2, v2 = (int(next(out)) for _ in range(4))
            if t1 == t2: sys.exit(1)
            add.append((ids[(t1, v1)], ids[(t2, v2)]))
        if not b.is_tree(nv, edges + add) or b.mis(nv, edges + add) != b.best(k, ids, edges, nv)[0]: sys.exit(1)
except (StopIteration, KeyError, ValueError):
    sys.exit(1)
sys.exit(0 if next(out, None) is None else 1)
