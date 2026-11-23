import sys
inp=list(map(int,open(sys.argv[1]).read().split())); out=list(map(int,open(sys.argv[2]).read().split())); ans=int(open(sys.argv[3]).read().split()[0])
n=inp[0]; xy=[(inp[1+2*i],inp[2+2*i]) for i in range(n)]; c=inp[1+2*n:1+3*n]; k=inp[1+3*n:1+4*n]
o=iter(out); tot=next(o); assert tot==ans,("total",tot,ans)
v=next(o); pl=[next(o) for _ in range(v)]; e=next(o); ed=[(next(o),next(o)) for _ in range(e)]
assert next(o,None) is None
assert len(set(pl))==v and all(1<=x<=n for x in pl)
cost=sum(c[x-1] for x in pl); par=list(range(n+1))
def f(x):
    while par[x]!=x: par[x]=par[par[x]]; x=par[x]
    return x
for x in pl: par[f(x)]=f(0)
seen=set()
for a,b in ed:
    assert a!=b and 1<=a<=n and 1<=b<=n and (min(a,b),max(a,b)) not in seen; seen.add((min(a,b),max(a,b)))
    cost+=(k[a-1]+k[b-1])*(abs(xy[a-1][0]-xy[b-1][0])+abs(xy[a-1][1]-xy[b-1][1])); par[f(a)]=f(b)
assert cost==tot,("cost",cost,tot)
assert all(f(i)==f(0) for i in range(1,n+1)), "unpowered"
