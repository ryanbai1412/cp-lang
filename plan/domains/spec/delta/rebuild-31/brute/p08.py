def gen(r):
    T=r.randint(1,3);out=[str(T)]
    for _ in range(T):
        q=r.randint(1,6);out.append(f"{r.randint(1,60)} {q}")
        for _ in range(q): out.append("2" if r.random()<.2 else f"1 {r.randint(1,30)}")
    return '\n'.join(out)+'\n'
def d(n):
    c=1;p=2
    while p*p<=n:
        e=0
        while n%p==0: n//=p;e+=1
        c*=e+1;p+=1
    return c*(2 if n>1 else 1)
def solve(inp):
    it=iter(inp.split());T=int(next(it));res=[]
    for _ in range(T):
        n0=int(next(it));q=int(next(it));n=n0
        for _ in range(q):
            if next(it)=='2': n=n0;continue
            n*=int(next(it));res.append('YES' if n%d(n)==0 else 'NO')
    return '\n'.join(res)
