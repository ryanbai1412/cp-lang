def gen(r):
    n=r.randint(1,12);return f"{n}\n"+' '.join(str(r.choice([r.randint(0,7),r.randint(0,10**9)])) for _ in range(n))+'\n'
def solve(inp,sq=False):
    v=list(map(int,inp.split()));n=v[0];a=v[1:];M=998244353;s=0
    for l in range(n):
        x=0
        for r in range(l,n):
            x^=a[r];w=r-l+1;s+=x*(w*w if sq else w)
    return str(s%M)
