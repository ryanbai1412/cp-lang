def gen(r):
    T=r.randint(1,3);return f"{T}\n"+'\n'.join(f"{r.randint(3,5)} {r.randint(3,5)} {r.randint(1,30)}" for _ in range(T))+'\n'
def feasible(n,m,k,H,V):
    # states: (vertex, last colour) reachable after exact steps
    def edges(x,y):
        if y+1<m: yield (x,y+1),H[x][y]
        if y>0: yield (x,y-1),H[x][y-1]
        if x+1<n: yield (x+1,y),V[x][y]
        if x>0: yield (x-1,y),V[x-1][y]
    cur=frozenset({((0,0),None)});seen={cur:0};t=0
    while t<k:
        cur=frozenset((p,c) for (v,l) in cur for p,c in edges(*v) if c!=l);t+=1
        if cur in seen:
            per=t-seen[cur];rem=(k-t)%per
            for _ in range(rem): cur=frozenset((p,c) for (v,l) in cur for p,c in edges(*v) if c!=l)
            break
        seen[cur]=t
    return any(v==(n-1,m-1) for v,_ in cur)
def check(inp,out):
    it=iter(inp.split());T=int(next(it));toks=out.split();pos=0
    for _ in range(T):
        n,m,k=int(next(it)),int(next(it)),int(next(it))
        poss=k>=n+m-2 and (k-n-m)%2==0
        if pos>=len(toks): return False
        w=toks[pos].upper();pos+=1
        if w=='NO':
            if poss: return False
            continue
        if not poss: return False
        if pos+n*(m-1)+(n-1)*m>len(toks): return False
        H=[toks[pos+i*(m-1):pos+(i+1)*(m-1)] for i in range(n)];pos+=n*(m-1)
        V=[toks[pos+i*m:pos+(i+1)*m] for i in range(n-1)];pos+=(n-1)*m
        if not feasible(n,m,k,H,V): return False
    return pos==len(toks)
