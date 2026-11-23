from collections import deque
def gen(r):
    T=r.randint(1,3);out=[str(T)]
    for _ in range(T):
        n=r.randint(1,9);k=r.randint(1,n);out+=[f"{n} {k}",' '.join(map(str,sorted(r.sample(range(1,n+1),k))))]
        for v in range(2,n+1): out.append(f"{r.randint(1,v-1)} {v}" if r.random()<.5 else f"{v} {r.randint(1,v-1)}")
    return '\n'.join(out)+'\n'
def solve(inp):
    it=iter(inp.split());T=int(next(it));res=[]
    for _ in range(T):
        n=int(next(it));k=int(next(it));mk=[int(next(it)) for _ in range(k)];adj=[[] for _ in range(n+1)]
        for _ in range(n-1):
            u=int(next(it));v=int(next(it));adj[u].append(v);adj[v].append(u)
        best=10**9
        for s in range(1,n+1):
            d=[-1]*(n+1);d[s]=0;q=deque([s])
            while q:
                x=q.popleft()
                for y in adj[x]:
                    if d[y]<0: d[y]=d[x]+1;q.append(y)
            best=min(best,max(d[a] for a in mk))
        res.append(str(best))
    return '\n'.join(res)
