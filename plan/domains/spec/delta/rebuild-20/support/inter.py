import subprocess, random, sys, os
FULL = os.environ.get("FULL") == "1"
def run(cmd, prob, seed, maxn, t):
    rnd = random.Random(seed)
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    toks = []
    def tok():
        while not toks:
            line = p.stdout.readline()
            if not line: raise EOFError("program closed")
            toks.extend(line.split())
        return toks.pop(0)
    def say(*a): p.stdin.write(" ".join(map(str, a)) + "\n"); p.stdin.flush()
    say(t); qmax = 0
    for _ in range(t):
        n = maxn if FULL else rnd.randint(1, maxn)
        if prob == "1903E":
            V = rnd.choice([3, 10, 10**9])
            s = (rnd.randint(0, V), rnd.randint(0, V)); pts = [(rnd.randint(0, V), rnd.randint(0, V)) for _ in range(n)]
            say(n); say(*s)
            for q in pts: say(*q)
            who = tok(); assert who in ("First", "Second"), who
            pf = who == "First"; used = [False]*n; prev = s; tot = 0; free = list(range(1, n+1)); posf = {k: k-1 for k in free}
            for turn in range(n):
                if (turn % 2 == 0) == pf: j = int(tok())
                else:
                    j = free[rnd.randrange(len(free))]; say(j)
                assert 1 <= j <= n and not used[j-1], j
                used[j-1] = True; i0 = posf[j]; last = free[-1]; free[i0] = last; posf[last] = i0; free.pop(); q = pts[j-1]; tot += (q[0]-prev[0])**2 + (q[1]-prev[1])**2; prev = q
            assert (tot % 2 == 0) == pf, "lost"
        else:
            a = list(range(1, n+1)); rnd.shuffle(a); x = rnd.randint(1, n); say(n); q = 0
            while True:
                op = tok()
                if op == "!":
                    got = [int(tok()) for _ in range(n)]; assert got == a, (got, a); break
                assert op == "?"; i = int(tok()); q += 1; assert q <= 40*n and 1 <= i <= n, (q, n)
                v = a[i-1]
                if v > x: say(">"); x += 1
                elif v < x: say("<"); x -= 1
                else: say("=")
            qmax = max(qmax, q / n)
    p.stdin.close(); rc = p.wait(); assert rc == 0, rc
    return qmax
if __name__ == "__main__":
    cmd, prob, seeds, maxn, t = sys.argv[1].split(), sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    worst = 0; fails = 0
    for sd in range(seeds):
        try: worst = max(worst, run(cmd, prob, sd, maxn, t))
        except Exception as e:
            fails += 1
            if fails <= 3: print("FAIL seed", sd, repr(e))
    print(prob, "runs", seeds, "fails", fails, "max queries/n %.2f" % worst)
