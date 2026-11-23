import sys, random, itertools
from math import comb
def b1936B(inp):
    it=inp.split(); t=int(it[0]); p=1; out=[]
    for _ in range(t):
        n=int(it[p]); s=list(it[p+1]); p+=2; res=[]
        for i in range(n):
            g=s[:]; pos=i; tm=0
            while 0<=pos<n:
                c=g[pos]; g[pos]='>' if c=='<' else '<'; pos+= -1 if c=='<' else 1; tm+=1
            res.append(tm)
        out.append(' '.join(map(str,res)))
    return '\n'.join(out)
def g1936B(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(1,8); L+= [str(n), ''.join(r.choice('<>') for _ in range(n))]
    return '\n'.join(L)
def b1943B(inp):
    it=inp.split(); t=int(it[0]); p=1; out=[]
    for _ in range(t):
        n,q=int(it[p]),int(it[p+1]); s=it[p+2]; p+=3
        for _ in range(q):
            l,r=int(it[p])-1,int(it[p+1]); p+=2; u=s[l:r]; tot=0
            for k in range(2,len(u)+1):
                if any(u[i:i+k]!=u[i:i+k][::-1] for i in range(len(u)-k+1)): tot+=k
            out.append(str(tot))
    return '\n'.join(out)
def g1943B(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(2,9); q=r.randint(1,6); al=r.choice(['ab','abc','a','aab'])
        L.append(f'{n} {q}'); L.append(''.join(r.choice(al) for _ in range(n)))
        for _ in range(q):
            l=r.randint(1,n-1); rr=r.randint(l+1,n); L.append(f'{l} {rr}')
    return '\n'.join(L)
def b1945F(inp):
    it=list(map(int,inp.split())); t=it[0]; p=1; out=[]
    for _ in range(t):
        n=it[p]; v=it[p+1:p+1+n]; pr=it[p+1+n:p+1+2*n]; p+=1+2*n; best=(0,0)
        for k in range(1,n+1):
            z=set(x-1 for x in pr[:k-1]); av=sorted([v[i] for i in range(n) if i not in z],reverse=True)
            if len(av)<k: continue
            s=k*av[k-1]
            if s>best[0]: best=(s,k)
        out.append(f'{best[0]} {best[1]}')
    return '\n'.join(out)
def g1945F(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(1,8); L+=[str(n),' '.join(str(r.randint(1,6)) for _ in range(n))]; pp=list(range(1,n+1)); r.shuffle(pp); L.append(' '.join(map(str,pp)))
    return '\n'.join(L)
def b1946D(inp):
    it=list(map(int,inp.split())); t=it[0]; p=1; out=[]
    for _ in range(t):
        n,x=it[p],it[p+1]; a=it[p+2:p+2+n]; p+=2+n; best=-1
        for mask in range(1<<(n-1)):
            segs=[];cur=0
            for i in range(n):
                cur^=a[i]
                if i==n-1 or mask>>i&1: segs.append(cur);cur=0
            o=0
            for s in segs:o|=s
            if o<=x: best=max(best,len(segs))
        out.append(str(best))
    return '\n'.join(out)
def g1946D(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(1,8); L+=[f'{n} {r.randint(0,15)}',' '.join(str(r.randint(0,15)) for _ in range(n))]
    return '\n'.join(L)
def b1946E(inp):
    it=list(map(int,inp.split())); t=it[0]; p=1; out=[]
    for _ in range(t):
        n,m1,m2=it[p:p+3]; P=it[p+3:p+3+m1]; S=it[p+3+m1:p+3+m1+m2]; p+=3+m1+m2; c=0
        for perm in itertools.permutations(range(1,n+1)):
            pm=[i+1 for i in range(n) if perm[i]==max(perm[:i+1])]; sm=[i+1 for i in range(n) if perm[i]==max(perm[i:])]
            if pm==P and sm==S: c+=1
        out.append(str(c%(10**9+7)))
    return '\n'.join(out)
def g1946E(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(1,7)
        if r.random()<0.7:
            perm=list(range(1,n+1)); r.shuffle(perm)
            P=[i+1 for i in range(n) if perm[i]==max(perm[:i+1])]; S=[i+1 for i in range(n) if perm[i]==max(perm[i:])]
        else:
            P=sorted(r.sample(range(1,n+1),r.randint(1,n))); S=sorted(r.sample(range(1,n+1),r.randint(1,n)))
        L+=[f'{n} {len(P)} {len(S)}',' '.join(map(str,P)),' '.join(map(str,S))]
    return '\n'.join(L)
def b1949C(inp):
    it=list(map(int,inp.split())); n=it[0]; adj=[[] for _ in range(n)]
    for i in range(n-1): u,v=it[1+2*i]-1,it[2+2*i]-1; adj[u].append(v); adj[v].append(u)
    start=tuple([1]*n); seen={start}; st=[start]
    while st:
        s=st.pop()
        if max(s)==n: return 'YES'
        for u in range(n):
            if s[u]==0: continue
            for v in adj[u]:
                if s[v]>0 and s[u]<=s[v]:
                    ns=list(s); ns[v]+=ns[u]; ns[u]=0; ns=tuple(ns)
                    if ns not in seen: seen.add(ns); st.append(ns)
    return 'NO'
def g1949C(r):
    n=r.randint(1,9); L=[str(n)]
    for i in range(2,n+1): L.append(f'{r.randint(1,i-1)} {i}')
    return '\n'.join(L)
def b1949F(inp): return None
def g1949F(r):
    n=r.randint(2,6); m=r.randint(1,5); L=[f'{n} {m}']
    for _ in range(n):
        k=r.randint(0,m); L.append(' '.join(map(str,[k]+r.sample(range(1,m+1),k))))
    return '\n'.join(L)
def chk1949F(inp,out):
    it=list(map(int,inp.split())); n,m=it[0],it[1]; p=2; S=[]
    for _ in range(n): k=it[p]; S.append(set(it[p+1:p+1+k])); p+=1+k
    exists=any((S[a]&S[b]) and S[a]-S[b] and S[b]-S[a] for a in range(n) for b in range(n))
    o=out.split()
    if not exists: return o==['NO']
    if len(o)!=3 or o[0]!='YES': return False
    a,b=int(o[1])-1,int(o[2])-1
    return a!=b and bool(S[a]&S[b]) and bool(S[a]-S[b]) and bool(S[b]-S[a])
def g1948E(r):
    t=r.randint(1,3); return '\n'.join([str(t)]+[f'{n} {r.randint(1,2*n)}' for n in [r.randint(2,12) for _ in range(t)]])
def chk1948E(inp,out):
    it=list(map(int,inp.split())); t=it[0]; o=list(map(int,out.split())); p=0
    for i in range(t):
        n,k=it[1+2*i],it[2+2*i]; a=o[p:p+n]; q=o[p+n]; c=o[p+n+1:p+2*n+1]; p+=2*n+1
        if sorted(a)!=list(range(1,n+1)) or not all(1<=x<=q for x in c): return False
        for u in range(n):
            for v in range(n):
                if c[u]==c[v] and abs(u-v)+abs(a[u]-a[v])>k: return False
        # optimal q = ceil(n/k)
        if q!=-(-n//k): return False
    return p==len(o)
def b1942D(inp):
    it=list(map(int,inp.split())); t=it[0]; p=1; out=[]
    for _ in range(t):
        n,k=it[p],it[p+1]; p+=2; a={}
        for i in range(1,n+1):
            for j in range(i,n+1): a[i,j]=it[p]; p+=1
        vals=[]
        for mask in range(1<<n):
            s=0; i=0
            while i<n:
                if mask>>i&1:
                    j=i
                    while j+1<n and mask>>(j+1)&1: j+=1
                    s+=a[i+1,j+1]; i=j+1
                else: i+=1
            vals.append(s)
        vals.sort(reverse=True); out.append(' '.join(map(str,vals[:k])))
    return '\n'.join(out)
def g1942D(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(1,6); k=r.randint(1,2**n); L.append(f'{n} {k}')
        for i in range(1,n+1): L.append(' '.join(str(r.randint(-5,5)) for _ in range(i,n+1)))
    return '\n'.join(L)
def b1941G(inp):
    import heapq
    it=list(map(int,inp.split())); t=it[0]; p=1; out=[]
    for _ in range(t):
        n,m=it[p],it[p+1]; p+=2; adj=[[] for _ in range(n+1)]
        for _ in range(m): u,v,c=it[p:p+3]; p+=3; adj[u].append((v,c)); adj[v].append((u,c))
        b,e=it[p],it[p+1]; p+=2
        if b==e: out.append('0'); continue
        INF=10**9; d={}; pq=[]
        for v,c in adj[b]: 
            if d.get((v,c),INF)>1: d[v,c]=1; heapq.heappush(pq,(1,v,c))
        best=INF
        while pq:
            dd,u,c=heapq.heappop(pq)
            if d[u,c]<dd: continue
            if u==e: best=min(best,dd)
            for v,c2 in adj[u]:
                nd=dd+(c2!=c)
                if d.get((v,c2),INF)>nd: d[v,c2]=nd; heapq.heappush(pq,(nd,v,c2))
        out.append(str(best))
    return '\n'.join(out)
def g1941G(r):
    t=r.randint(1,3); L=[str(t)]
    for _ in range(t):
        n=r.randint(2,7); edges=[(r.randint(1,i-1),i) for i in range(2,n+1)]
        allp=[(u,v) for u in range(1,n+1) for v in range(u+1,n+1) if (u,v) not in edges]
        edges+=r.sample(allp,r.randint(0,min(3,len(allp))))
        # colour: each colour must be connected: grow colours by assigning each edge a colour of an adjacent edge or new
        col={}; nc=0
        for (u,v) in edges:
            cand=[col[f] for f in col if set(f)&{u,v}]
            if cand and r.random()<0.6: col[(u,v)]=r.choice(cand)
            else: nc+=1; col[(u,v)]=r.choice([nc, nc+100000])
        L.append(f'{n} {len(edges)}')
        for (u,v) in edges: L.append(f'{u} {v} {col[(u,v)]}')
        L.append(f'{r.randint(1,n)} {r.randint(1,n)}')
    return '\n'.join(L)
