import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from importlib import util
spec = util.spec_from_file_location("b", __file__.rsplit("/", 1)[0] + "/brute.py")
inp, out = open(sys.argv[1]).read().split(), open(sys.argv[2]).read().split()
def composite(x): return x > 3 and any(x % d == 0 for d in range(2, int(x ** 0.5) + 1))
src = open(__file__.rsplit("/", 1)[0] + "/brute.py").read().split("it = iter")[0]
ns = {}; exec(src, ns)
pos = 0
for n in map(int, inp[1:]):
    exists = ns["match"](n) is not None
    if out[pos] == "-1":
        if exists: sys.exit(1)
        pos += 1; continue
    p = list(map(int, out[pos:pos + n])); pos += n
    if sorted(p) != list(range(1, n + 1)) or not all(composite((i + 1) * v) for i, v in enumerate(p)): sys.exit(1)
sys.exit(0 if pos == len(out) else 1)
