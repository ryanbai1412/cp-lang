def gen(r,rev=False):
    T=r.randint(1,2);out=[str(T)]
    for _ in range(T):
        q=r.randint(1,10);out.append(str(q));sz=1
        for _ in range(q):
            x=r.random()
            if x<.4: out.append(f"1 {r.randint(1,sz)}");sz+=1
            elif x<.8 or not rev: out.append(f"2 {r.randint(1,sz)} {r.randint(-10**9,10**9)}")
            else: out.append(f"3 {r.randint(1,sz)}")
    return '\n'.join(out)+'\n'
def solve(inp):
    it=iter(inp.split());T=int(next(it));res=[]
    for _ in range(T):
        q=int(next(it));par=[0,0];val=[0,0]
        for _ in range(q):
            t=int(next(it))
            if t==1: par.append(int(next(it)));val.append(0)
            elif t==2:
                v=int(next(it));x=int(next(it))
                for u in range(1,len(par)):
                    w=u
                    while w and w!=v: w=par[w]
                    if w==v: val[u]+=x
            else: res.append(str(val[int(next(it))]))
        res.append(' '.join(map(str,val[1:])))
    return '\n'.join(res)
