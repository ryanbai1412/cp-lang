import sys
inp=open(sys.argv[1]).read().split(); out=open(sys.argv[2]).read().split()
it=iter(inp); t=int(next(it)); o=iter(out)
for _ in range(t):
    n=int(next(it)); m=int(next(it)); adj=[set() for _ in range(n+1)]
    for _ in range(m):
        u=int(next(it)); v=int(next(it)); adj[u].add(v); adj[v].add(u)
    k=int(next(o)); c=[int(next(o)) for _ in range(k)]
    assert 1<=k<=n//2 and len(set(c))==k and all(1<=x<=n for x in c), "bad set"
    s=set(c)
    for v in range(1,n+1):
        assert v in s or adj[v]&s, f"undominated {v}"
assert next(o,None) is None, "extra output"
