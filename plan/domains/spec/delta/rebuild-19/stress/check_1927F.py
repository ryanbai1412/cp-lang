import sys
inp=open(sys.argv[1]).read().split(); out=open(sys.argv[2]).read().split(); exp=open(sys.argv[3]).read().split('\n')
# exp: first token of each answer's first line = b
it=iter(inp); t=int(next(it)); o=iter(out); el=[l for l in exp if l.strip()]
for tc in range(t):
    n,m=int(next(it)),int(next(it)); W={}
    for _ in range(m):
        u,v,w=int(next(it)),int(next(it)),int(next(it)); W[(u,v)]=W[(v,u)]=w
    b,k=int(next(o)),int(next(o)); cyc=[int(next(o)) for _ in range(k)]
    eb=int(el[2*tc].split()[0])
    assert b==eb, (tc,'b',b,eb)
    assert k>=3 and len(set(cyc))==k, (tc,'not simple')
    ws=[W.get((cyc[i],cyc[(i+1)%k])) for i in range(k)]
    assert None not in ws, (tc,'missing edge')
    assert min(ws)==b, (tc,'min')
print("OK")
