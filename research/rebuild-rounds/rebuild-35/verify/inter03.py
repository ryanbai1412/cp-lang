import sys, random
x=int(open(sys.argv[1]).read().split()[0]); rng=random.Random(x*7+1)
q=0
def bad(m): print(-1, flush=True); sys.stderr.write(m+"\n"); sys.exit(1)
while True:
    line=sys.stdin.readline()
    if not line: bad("eof")
    t=line.split()
    if t[0]=='?':
        q+=1
        a=list(map(int,t[1:]))
        if q>2 or len(a)!=100 or len(set(a))!=100 or not all(0<=v<2**14 for v in a): bad("bad query")
        print(x^rng.choice(a), flush=True)
    elif t[0]=='!':
        sys.exit(0 if int(t[1])==x else 1)
    else: bad("garbage")
