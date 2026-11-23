import itertools
def gen(r):
    return f"{r.randint(1,3)} {r.randint(1,3)}\n"
def solve(inp):
    N,K=map(int,inp.split());c=0
    sets=range(1<<K)
    def child(a,b): return all((not a>>x&1) or (x+1<K and b>>(x+1)&1) for x in range(K))
    for arr in itertools.product(sets,repeat=N):
        if any(all(child(p[i],p[i+1]) for i in range(N-1)) for p in itertools.permutations(arr)): c+=1
    return str(c%998244353)
