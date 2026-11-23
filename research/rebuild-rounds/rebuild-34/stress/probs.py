import random, itertools
from functools import lru_cache
from collections import deque

def topo_ok(n, arcs):
    indeg=[0]*(n+1); adj=[[] for _ in range(n+1)]
    for u,v in arcs: adj[u].append(v); indeg[v]+=1
    q=[i for i in range(1,n+1) if indeg[i]==0]; c=0
    while q:
        u=q.pop(); c+=1
        for v in adj[u]:
            indeg[v]-=1
            if indeg[v]==0: q.append(v)
    return c==n

# ---------- 01 1358D
def g01(r):
    n=r.randint(1,5); d=[r.randint(1,4) for _ in range(n)]; x=r.randint(1,sum(d))
    return f"{n} {x}\n{' '.join(map(str,d))}\n"
def b01(s):
    t=list(map(int,s.split())); n,x=t[0],t[1]; d=t[2:2+n]
    days=[j for m in d for j in range(1,m+1)]*2
    L=len(days)//2
    return str(max(sum(days[i:i+x]) for i in range(L)))
# ---------- 02 1324F
def tree(r,n): return [(r.randint(1,i-1),i) for i in range(2,n+1)]
def g02(r):
    n=r.randint(2,8); e=tree(r,n); p=list(range(1,n+1)); r.shuffle(p)
    return f"{n}\n{' '.join(str(r.randint(0,1)) for _ in range(n))}\n"+"".join(f"{p[u-1]} {p[v-1]}\n" for u,v in e)
def best_conn(n,w,edges):
    adj=[set() for _ in range(n)]
    for u,v in edges: adj[u-1].add(v-1); adj[v-1].add(u-1)
    res=[None]*n
    for mask in range(1,1<<n):
        vs=[i for i in range(n) if mask>>i&1]
        seen={vs[0]}; st=[vs[0]]
        while st:
            u=st.pop()
            for v in adj[u]:
                if mask>>v&1 and v not in seen: seen.add(v); st.append(v)
        if len(seen)!=len(vs): continue
        s=sum(w[i] for i in vs)
        for i in vs:
            if res[i] is None or s>res[i]: res[i]=s
    return ' '.join(map(str,res))
def b02(s):
    t=list(map(int,s.split())); n=t[0]; a=t[1:1+n]; e=t[1+n:]
    return best_conn(n,[1 if c else -1 for c in a],list(zip(e[::2],e[1::2])))
def g02r(r):
    n=r.randint(2,8); e=tree(r,n)
    return f"{n}\n{' '.join(str(r.randint(-10,10)) for _ in range(n))}\n"+"".join(f"{u} {v}\n" for u,v in e)
def b02r(s):
    t=list(map(int,s.split())); n=t[0]; a=t[1:1+n]; e=t[1+n:]
    return best_conn(n,a,list(zip(e[::2],e[1::2])))
# ---------- 03 1385E (checker)
def g03(r):
    T=r.randint(1,3); out=f"{T}\n"
    for _ in range(T):
        n=r.randint(2,6); pairs=[(i,j) for i in range(1,n+1) for j in range(i+1,n+1)]; r.shuffle(pairs)
        m=r.randint(1,len(pairs)); out+=f"{n} {m}\n"
        for u,v in pairs[:m]:
            if r.random()<0.5: u,v=v,u
            out+=f"{r.randint(0,1)} {u} {v}\n"
    return out
def c03(s,got):
    try: return _c03(s,got)
    except Exception: return False
def _c03(s,got):
    t=list(map(int,s.split())); g=got.split(); p=0; gi=0
    T=t[p]; p+=1
    for _ in range(T):
        n,m=t[p],t[p+1]; p+=2; es=[]
        for _ in range(m): es.append(tuple(t[p:p+3])); p+=3
        feas=topo_ok(n,[(x,y) for ty,x,y in es if ty==1])
        w=g[gi]; gi+=1
        if w!=("YES" if feas else "NO"): return False
        if not feas: continue
        arcs=[]
        for ty,x,y in es:
            u,v=int(g[gi]),int(g[gi+1]); gi+=2
            if ty==1 and (u,v)!=(x,y): return False
            if ty==0 and (u,v) not in ((x,y),(y,x)): return False
            arcs.append((u,v))
        if not topo_ok(n,arcs): return False
    return gi==len(g)
# ---------- 04 1343D
def g04(r):
    T=r.randint(1,3); out=f"{T}\n"
    for _ in range(T):
        n=2*r.randint(1,4); k=r.randint(1,6); out+=f"{n} {k}\n{' '.join(str(r.randint(1,k)) for _ in range(n))}\n"
    return out
def b04(s):
    t=list(map(int,s.split())); p=1; res=[]
    for _ in range(t[0]):
        n,k=t[p],t[p+1]; a=t[p+2:p+2+n]; p+=2+n; best=10**9
        for x in range(2,2*k+1):
            c=0
            for i in range(n//2):
                u,v=a[i],a[n-1-i]
                if u+v==x: continue
                c+= 1 if (1<=x-u<=k or 1<=x-v<=k) else 2
            best=min(best,c)
        res.append(str(best))
    return '\n'.join(res)
# ---------- 05 1370D
def g05(r):
    n=r.randint(2,9); k=r.randint(2,n); return f"{n} {k}\n{' '.join(str(r.randint(1,10)) for _ in range(n))}\n"
def b05(s):
    t=list(map(int,s.split())); n,k=t[0],t[1]; a=t[2:]
    return str(min(min(max(c[0::2]),max(c[1::2])) for c in itertools.combinations(a,k)))
# ---------- 06 1430D
def g06(r):
    T=r.randint(1,3); out=f"{T}\n"
    for _ in range(T):
        n=r.randint(1,10); out+=f"{n}\n{''.join(r.choice('01') for _ in range(n))}\n"
    return out
@lru_cache(None)
def ops06(s):
    if not s: return 0
    best=0
    for i in range(len(s)):
        u=s[:i]+s[i+1:]
        if u:
            j=0
            while j<len(u) and u[j]==u[0]: j+=1
            u=u[j:]
        best=max(best,1+ops06(u))
    return best
def b06(s):
    t=s.split(); return '\n'.join(str(ops06(t[2+2*i])) for i in range(int(t[0])))
# ---------- 07 1404C
def g07(r):
    n=r.randint(1,8); q=r.randint(1,5); a=[r.randint(1,n) for _ in range(n)]
    qs=[]
    for _ in range(q):
        x=r.randint(0,n-1); y=r.randint(0,n-1-x); qs.append(f"{x} {y}\n")
    return f"{n} {q}\n{' '.join(map(str,a))}\n"+''.join(qs)
@lru_cache(None)
def w07(a):
    best=0
    for i,v in enumerate(a):
        if v==i+1: best=max(best,1+w07(a[:i]+a[i+1:]))
    return best
def b07(s):
    t=list(map(int,s.split())); n,q=t[0],t[1]; a=t[2:2+n]; res=[]
    for j in range(q):
        x,y=t[2+n+2*j],t[3+n+2*j]
        b=tuple([n+1]*x+a[x:n-y]+[n+1]*y); res.append(str(w07(b)))
    return '\n'.join(res)
# ---------- 08 1359E
def g08(r): return f"{r.randint(1,9)} {r.randint(1,4)}\n"
def b08(s):
    n,k=map(int,s.split()); c=0
    for arr in itertools.combinations(range(1,n+1),k):
        ok=True
        for x in range(0,850):
            vals=set()
            for p in itertools.permutations(arr):
                y=x
                for v in p: y%=v
                vals.add(y)
                if len(vals)>1: break
            if len(vals)>1: ok=False; break
        c+=ok
    return str(c%998244353)
# ---------- 09 1365D
def g09(r):
    T=r.randint(1,3); out=f"{T}\n"
    for _ in range(T):
        n=r.randint(1,3); m=r.randint(1,3); g=[[r.choice('..#GB') for _ in range(m)] for _ in range(n)]; g[-1][-1]='.'
        out+=f"{n} {m}\n"+''.join(''.join(row)+'\n' for row in g)
    return out
def solve09(n,m,g):
    empt=[(i,j) for i in range(n) for j in range(m) if g[i][j]=='.']
    for mask in range(1<<len(empt)):
        h=[row[:] for row in g]
        for b,(i,j) in enumerate(empt):
            if mask>>b&1: h[i][j]='#'
        reach=set()
        if h[n-1][m-1]!='#':
            dq=deque([(n-1,m-1)]); reach.add((n-1,m-1))
            while dq:
                i,j=dq.popleft()
                for di,dj in ((1,0),(-1,0),(0,1),(0,-1)):
                    a,b2=i+di,j+dj
                    if 0<=a<n and 0<=b2<m and h[a][b2]!='#' and (a,b2) not in reach: reach.add((a,b2)); dq.append((a,b2))
        if all(((i,j) in reach)==(g[i][j]=='G') for i in range(n) for j in range(m) if g[i][j] in 'GB'): return "Yes"
    return "No"
def b09(s):
    t=s.split(); p=1; res=[]
    for _ in range(int(t[0])):
        n,m=int(t[p]),int(t[p+1]); g=[list(t[p+2+i]) for i in range(n)]; p+=2+n
        res.append(solve09(n,m,g))
    return '\n'.join(res)
# ---------- 10 1526C2
def g10(r):
    n=r.randint(1,12); return f"{n}\n{' '.join(str(r.randint(-6,6)) for _ in range(n))}\n"
def subsets10(a,h0):
    best=(-1,0)
    for mask in range(1<<len(a)):
        h=h0; ok=True
        for i,v in enumerate(a):
            if mask>>i&1:
                h+=v
                if h<0: ok=False; break
        if ok: best=max(best,(bin(mask).count('1'),h))
    return best
def b10(s):
    t=list(map(int,s.split())); return str(subsets10(t[1:],0)[0])
def g10r(r):
    n=r.randint(1,12); return f"{n}\n{r.randint(0,6)}\n{' '.join(str(r.randint(-6,6)) for _ in range(n))}\n"
def b10r(s):
    t=list(map(int,s.split())); c,h=subsets10(t[2:],t[1]); return f"{c} {h}"

P={'01-1358D':(g01,b01),'02-1324F':(g02,b02),'02-1324F-rev':(g02r,b02r),'03-1385E':(g03,None),
   '04-1343D':(g04,b04),'05-1370D':(g05,b05),'06-1430D':(g06,b06),'07-1404C':(g07,b07),'08-1359E':(g08,b08),
   '09-1365D':(g09,b09),'10-1526C2':(g10,b10),'10-1526C2-rev':(g10r,b10r)}
CHK={'03-1385E':c03}
