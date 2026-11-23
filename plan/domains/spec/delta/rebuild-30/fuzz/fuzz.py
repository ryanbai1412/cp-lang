import random, subprocess, sys, os
from concurrent.futures import ThreadPoolExecutor
from brute import *
HERE = os.path.dirname(os.path.abspath(__file__))
D = os.path.dirname(HERE)
CPL = os.environ.get('CPL', 'cpl'); BIN = os.environ.get('BIN', os.path.join(HERE, 'bin'))
G = {
 '01-1918D': lambda r: multi(g1918, s1918, r),
 '02-1905D': lambda r: multi(g1905, s1905, r),
 '03-1915G': lambda r: multi(g1915, s1915, r),
 '03-1915G-rev': lambda r: multi(g1915, s1915rev, r),
 '04-1924B': gen1924,
 '05-1930D2': lambda r: multi(g1930, s1930, r),
 '07-1929E': lambda r: multi(g1929, s1929, r),
 '07-1929E-rev': lambda r: multi(lambda rr: g1929(rr, True), s1929, r),
 '08-1925D': lambda r: multi(g1925, s1925, r),
 '09-1948D': lambda r: multi(g1948, s1948, r),
 '10-1923D': lambda r: multi(g1923, s1923, r),
}
norm = lambda s: [l.split() for l in s.strip().splitlines()]
def one(args):
    name, seed = args; r = random.Random(seed); inp, exp = G[name](r)
    a = subprocess.run([CPL, 'run', f'{D}/{name}.cp'], input=inp, capture_output=True, text=True, timeout=120)
    b = subprocess.run([f'{BIN}/{name}'], input=inp, capture_output=True, text=True, timeout=60)
    bad = []
    if norm(a.stdout) != norm(exp): bad.append('interp!=brute')
    if norm(b.stdout) != norm(exp): bad.append('native!=brute')
    if norm(a.stdout) != norm(b.stdout): bad.append('interp!=native')
    return seed, bad, inp, exp, a.stdout + a.stderr[-400:], b.stdout + b.stderr[-300:]
names = sys.argv[1:] or list(G)
for name in names:
    with ThreadPoolExecutor(16) as ex: res = list(ex.map(one, [(name, s) for s in range(200)]))
    fails = [x for x in res if x[1]]
    print(f"{name}: {200-len(fails)}/200 ok", flush=True)
    for seed, bad, inp, exp, ao, bo in sorted(fails, key=lambda x: len(x[2]))[:2]:
        print(f"  seed {seed} {bad}\n--in\n{inp}--exp\n{exp}--interp\n{ao}\n--native\n{bo}")
    open(os.path.join(HERE, f'{name}.fails'), 'w').write(repr([(x[0], x[1], x[2]) for x in fails]))
