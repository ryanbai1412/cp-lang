import sys, importlib.util, os
spec = importlib.util.spec_from_file_location("brute", os.path.join(os.path.dirname(__file__), "brute.py"))
b = importlib.util.module_from_spec(spec); spec.loader.exec_module(b)
out = open(sys.argv[2]).read().split(); pos = 0
for a, k in b.cases(open(sys.argv[1]).read()):
    want = b.best(a, k)[0]
    if out[pos] == "-1": got = b.value(a, k); pos += 1
    else:
        i, j = int(out[pos]) - 1, int(out[pos + 1]) - 1; pos += 2
        if not (0 <= i < len(a) and 0 <= j < len(a) and i != j): sys.exit(1)
        c = a[:]; c[i], c[j] = c[j], c[i]; got = b.value(c, k)
    if got != want: sys.exit(1)
sys.exit(0 if pos == len(out) else 1)
