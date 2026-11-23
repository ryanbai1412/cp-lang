import random, subprocess, os
from concurrent.futures import ThreadPoolExecutor
HERE = os.path.dirname(os.path.abspath(__file__))
D = os.path.dirname(HERE)
CPL = os.environ.get('CPL', 'cpl'); BIN = os.environ.get('BIN', os.path.join(HERE, 'bin'))
BIN06 = os.environ.get('BIN06', os.path.join(BIN, '06-1934C'))
def gen(r):
    t = r.randint(1, 3); cs = []
    for _ in range(t):
        n = r.randint(2, 6); m = r.randint(2, 6)
        cells = [(x, y) for x in range(1, n + 1) for y in range(1, m + 1)]
        a, b = r.sample(cells, 2); cs.append((n, m, a, b))
    return cs
def native(cs):
    p = subprocess.Popen([BIN06], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    p.stdin.write(f"{len(cs)}\n"); p.stdin.flush(); log = []
    for n, m, a, b in cs:
        p.stdin.write(f"{n} {m}\n"); p.stdin.flush(); q = 0
        while True:
            line = p.stdout.readline().split(); log.append(" ".join(line))
            if not line: return False, log
            x, y = int(line[1]), int(line[2])
            if line[0] == '!':
                if (x, y) not in (a, b): return False, log
                break
            q += 1
            if q > 4 or not (1 <= x <= n and 1 <= y <= m): return False, log
            d = min(abs(x - a[0]) + abs(y - a[1]), abs(x - b[0]) + abs(y - b[1]))
            p.stdin.write(f"{d}\n"); p.stdin.flush()
    p.stdin.close(); p.wait(); return True, log
def one(seed):
    cs = gen(random.Random(seed))
    hidden = f"{len(cs)}\n" + "".join(f"{n} {m}\n{a[0]} {a[1]}\n{b[0]} {b[1]}\n" for n, m, a, b in cs)
    r = subprocess.run([CPL, 'run', f'{D}/06-1934C.cp'], input=hidden, capture_output=True, text=True, timeout=60)
    ok_i = 'accepted' in r.stderr + r.stdout
    tr_i = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    ok_n, tr_n = native(cs)
    bad = [] if ok_i and ok_n and tr_i == tr_n else [f"interp_ok={ok_i} native_ok={ok_n} same={tr_i == tr_n}"]
    return seed, bad, hidden, r.stdout + r.stderr[-300:], tr_n
with ThreadPoolExecutor(16) as ex: res = list(ex.map(one, range(200)))
f = [x for x in res if x[1]]
print(f"06-1934C: {200 - len(f)}/200 ok")
for x in f[:2]: print(x)
