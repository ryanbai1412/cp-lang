import itertools
def gen(r):
    T=r.randint(1,3);out=[str(T)]
    for _ in range(T):
        n=r.randint(1,9);k=r.randint(0,n);out+=[f"{n} {k}",''.join(r.choice('01') for _ in range(n))]
    return '\n'.join(out)+'\n'
def runs(s,c):
    b=cur=0
    for ch in s:
        cur=cur+1 if ch==c else 0;b=max(b,cur)
    return b
def solve(inp):
    it=iter(inp.split());T=int(next(it));res=[]
    for _ in range(T):
        n=int(next(it));k=int(next(it));s=next(it);pairs=set()
        for cnt in range(k+1):
            for idx in itertools.combinations(range(n),cnt):
                t=list(s)
                for i in idx: t[i]='1' if t[i]=='0' else '0'
                t=''.join(t);pairs.add((runs(t,'0'),runs(t,'1')))
        res.append(' '.join(str(max(a*x+y for x,y in pairs)) for a in range(1,n+1)))
    return '\n'.join(res)
