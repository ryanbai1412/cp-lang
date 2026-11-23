from functools import lru_cache
def gen(r):
    n=r.randint(1,3);m=r.randint(1,6);k=r.randint(1,min(4,m))
    return f"{n} {m} {k}\n"+'\n'.join(' '.join(str(r.randint(1,20)) for _ in range(m)) for _ in range(n))+'\n'
def solve(inp):
    v=list(map(int,inp.split()));n,m,k=v[:3];a=[v[3+i*m:3+(i+1)*m] for i in range(n)]
    cells=[(i,j) for i in range(n) for j in range(m)]
    @lru_cache(None)
    def f(p,used):
        if p==m-k+1: return 0
        b=-1
        for idx,(i,j) in enumerate(cells):
            if p<=j<=p+k-1:
                val=0 if used>>idx&1 else a[i][j]
                b=max(b,val+f(p+1,used|1<<idx))
        return b
    return str(f(0,0))
