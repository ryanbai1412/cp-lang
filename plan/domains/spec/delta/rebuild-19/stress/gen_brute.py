import random, itertools
from collections import deque
def tree(r,n):
    return [(r.randint(1,i-1),i) for i in range(2,n+1)]
def relabel(r,n,es):
    p=list(range(1,n+1)); r.shuffle(p); return [(p[u-1],p[v-1]) if r.random()<.5 else (p[v-1],p[u-1]) for u,v in es]
P={}
# 1922D
def g(r):
    t=r.randint(1,3); s=[str(t)]
    for _ in range(t):
        n=r.randint(1,8); s+= [str(n),' '.join(str(r.randint(1,10)) for _ in range(n)),' '.join(str(r.randint(1,15)) for _ in range(n))]
    return '\n'.join(s)+'\n'
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n=int(next(it)); a=[int(next(it)) for _ in range(n)]; d=[int(next(it)) for _ in range(n)]
        al=list(range(n)); res=[]
        for _ in range(n):
            die=[al[i] for i in range(len(al)) if (a[al[i-1]] if i>0 else 0)+(a[al[i+1]] if i+1<len(al) else 0)>d[al[i]]]
            res.append(len(die)); al=[v for v in al if v not in die]
        o.append(' '.join(map(str,res)))
    return '\n'.join(o)
P['1922D']=(g,b)
# 1932F
def g(r):
    t=r.randint(1,3); s=[str(t)]
    for _ in range(t):
        n=r.randint(1,8); m=r.randint(1,6); s.append(f"{n} {m}")
        for _ in range(m):
            l=r.randint(1,n); s.append(f"{l} {r.randint(l,n)}")
    return '\n'.join(s)+'\n'
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n,m=int(next(it)),int(next(it)); c=[(int(next(it)),int(next(it))) for _ in range(m)]; best=0
        for mask in range(1<<n):
            pts=[i+1 for i in range(n) if mask>>i&1]; ok=True; fed=0
            for l,rr in c:
                k=sum(1 for p in pts if l<=p<=rr)
                if k>1: ok=False;break
                fed+=k
            if ok: best=max(best,fed)
        o.append(str(best))
    return '\n'.join(o)
P['1932F']=(g,b)
# 1923E
def g(r):
    t=r.randint(1,3); s=[str(t)]
    for _ in range(t):
        n=r.randint(2,8); s+=[str(n),' '.join(str(r.randint(1,min(n,3))) for _ in range(n))]
        s+=[f"{u} {v}" for u,v in relabel(r,n,tree(r,n))]
    return '\n'.join(s)+'\n'
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n=int(next(it)); c=[0]+[int(next(it)) for _ in range(n)]; adj=[[] for _ in range(n+1)]
        for _ in range(n-1):
            u,v=int(next(it)),int(next(it)); adj[u].append(v); adj[v].append(u)
        cnt=0
        for s0 in range(1,n+1):
            par={s0:0}; q=[s0]
            for u in q:
                for w in adj[u]:
                    if w not in par: par[w]=u; q.append(w)
            for e in range(s0+1,n+1):
                if c[e]!=c[s0]: continue
                v=par[e]; ok=True
                while v!=s0:
                    if c[v]==c[s0]: ok=False
                    v=par[v]
                cnt+=ok
        o.append(str(cnt))
    return '\n'.join(o)
P['1923E']=(g,b)
# 1928C
def g(r):
    t=r.randint(1,5); s=[str(t)]
    for _ in range(t):
        n=r.randint(2,60); s.append(f"{n} {r.randint(1,n-1)}")
    return '\n'.join(s)+'\n'
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n,xx=int(next(it)),int(next(it)); c=0
        for k in range(2,2*n+2):
            seq=list(range(1,k+1))+list(range(k-1,1,-1)); 
            if seq[(n-1)%len(seq)]==xx: c+=1
        o.append(str(c))
    return '\n'.join(o)
P['1928C']=(g,b)
# 1929D
def g(r):
    t=r.randint(1,3); s=[str(t)]
    for _ in range(t):
        n=r.randint(2,8); s.append(str(n)); s+=[f"{u} {v}" for u,v in relabel(r,n,tree(r,n))]
    return '\n'.join(s)+'\n'
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n=int(next(it)); adj=[[] for _ in range(n+1)]
        for _ in range(n-1):
            u,v=int(next(it)),int(next(it)); adj[u].append(v); adj[v].append(u)
        paths=[]
        for s0 in range(1,n+1):
            par={s0:0}; q=[s0]
            for u in q:
                for w in adj[u]:
                    if w not in par: par[w]=u; q.append(w)
            for e in range(s0,n+1):
                m=0; v=e
                while v: m|=1<<(v-1); v=par[v] if v!=s0 else 0
                paths.append(m)
        cnt=sum(1 for S in range(1<<n) if all(bin(S&p).count('1')<=2 for p in paths))
        o.append(str(cnt%998244353))
    return '\n'.join(o)
P['1929D']=(g,b)
# 1933F
def g(r):
    t=r.randint(1,3); s=[str(t)]
    for _ in range(t):
        n,m=r.randint(3,5),r.randint(3,5); s.append(f"{n} {m}")
        for i in range(n):
            s.append(' '.join('0' if (j==m-1 or (i,j)==(0,0)) else str(int(r.random()<.3)) for j in range(m)))
    return '\n'.join(s)+'\n'
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n,m=int(next(it)),int(next(it)); a=[[int(next(it)) for _ in range(m)] for _ in range(n)]
        rock=lambda i,j,tt: a[(i+tt)%n][j]==1
        D={(0,0,0):0}; q=deque([(0,0,0)]); ans=-1
        while q:
            x,y,tm=q.popleft(); dd=D[(x,y,tm)]
            if (x,y)==(n-1,m-1): ans=dd; break
            nx=[]
            nx.append(((x-1)%n,y))
            if not rock((x+1)%n,y,tm) and not rock((x+2)%n,y,tm): nx.append(((x+1)%n,y))
            if y+1<m and not rock((x+1)%n,y+1,tm): nx.append((x,y+1))
            for u,v in nx:
                k=(u,v,(tm+1)%n)
                if k not in D: D[k]=dd+1; q.append(k)
        o.append(str(ans))
    return '\n'.join(o)
P['1933F']=(g,b)
# 1930D1
def g(r):
    t=r.randint(1,3); s=[str(t)]
    for _ in range(t):
        n=r.randint(1,8); s+=[str(n),''.join(r.choice('01') for _ in range(n))]
    return '\n'.join(s)+'\n'
def f(p):
    m=len(p); best=m
    for q in range(1<<m):
        bits=[q>>i&1 for i in range(m)]; ok=True
        for i in range(m):
            want=int(p[i]); good=False
            for l in range(i+1):
                for rr in range(i,m):
                    L=rr-l+1; c=sum(bits[l:rr+1]) if want else L-sum(bits[l:rr+1])
                    if c>=(L+1)//2: good=True;break
                if good:break
            if not good: ok=False;break
        if ok: best=min(best,sum(bits))
    return best
def b(x):
    it=iter(x.split()); t=int(next(it)); o=[]
    for _ in range(t):
        n=int(next(it)); s=next(it)
        o.append(str(sum(f(s[i:j+1]) for i in range(n) for j in range(i,n))))
    return '\n'.join(o)
P['1930D1']=(g,b)
