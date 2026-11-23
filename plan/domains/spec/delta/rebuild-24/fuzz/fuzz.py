# Generators + brutes for 8 files; 200 seeds each compare `cpl run`, the C++ build and the brute (08: a checker plus exhaustive -1 check).
# usage: CPL=target/release/cpl BIN=<dir of built binaries> python3 fuzz.py [stem...]
import random, subprocess, sys, itertools, math, os
from concurrent.futures import ThreadPoolExecutor
D=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
C=os.environ.get('CPL','cpl')
B=os.environ['BIN']  # dir holding the g++ -O2 builds, one per file stem
def g01(r):
    t=r.randint(1,4); return f"{t}\n"+"".join(f"{r.randint(0,100)} {r.randint(0,100)}\n" for _ in range(t))
def b01(inp):
    v=list(map(int,inp.split())); out=[]
    for i in range(v[0]):
        x,y=v[1+2*i],v[2+2*i]; best=None
        for asg in itertools.product(range(3),repeat=8):
            a=sum(k+1 for k in range(8) if asg[k]==1); b=sum(k+1 for k in range(8) if asg[k]==2)
            if x>>a==y>>b:
                c=sum(2**(k+1) for k in range(8) if asg[k]); best=c if best is None else min(best,c)
        out.append(best)
    return out
def g02(r):
    t=r.randint(1,5); s=f"{t}\n"
    for _ in range(t):
        n=r.randint(1,60); s+=f"{n} {r.randint(1,n)}\n"
    return s
def b02(inp):
    v=list(map(int,inp.split())); out=[]
    for i in range(v[0]):
        n,k=v[1+2*i],v[2+2*i]
        def go(l,r_):
            if r_-l+1<k: return 0
            m=(l+r_)//2
            if (r_-l+1)%2==0: return go(l,m)+go(m+1,r_)
            return m+(go(l,m-1)+go(m+1,r_) if l!=r_ else 0)
        out.append(go(1,n))
    return out
def g04(r):
    t=r.randint(1,3); s=f"{t}\n"
    for _ in range(t):
        n=r.randint(1,9); k=r.randint(0,n)
        s+=f"{n} {k}\n"+"".join(r.choice("RB") for _ in range(n))+"\n"+" ".join(str(r.randint(1,10)) for _ in range(n))+"\n"
    return s
def b04(inp):
    tk=inp.split(); t=int(tk[0]); p=1; out=[]
    for _ in range(t):
        n,k=int(tk[p]),int(tk[p+1]); s=tk[p+2]; a=list(map(int,tk[p+3:p+3+n])); p+=3+n; best=None
        for mask in range(1<<n):
            col=[(mask>>i)&1 for i in range(n)]
            runs=sum(1 for i in range(n) if col[i] and (i==0 or not col[i-1]))
            if runs>k: continue
            pen=max([a[i] for i in range(n) if col[i]!=(s[i]=='B')],default=0)
            best=pen if best is None else min(best,pen)
        out.append(best)
    return out
def g05(r):
    t=r.randint(1,3); s=f"{t}\n"
    for _ in range(t):
        n=r.randint(1,8); s+=f"{n}\n"+" ".join(str(r.randint(1,5)) for _ in range(n))+"\n"
    return s
def b05(inp):
    tk=list(map(int,inp.split())); t=tk[0]; p=1; out=[]
    for _ in range(t):
        n=tk[p]; a=tk[p+1:p+1+n]; p+=1+n; best=None
        for mask in range(1<<n):
            res=[a[i] for i in range(n) if not mask>>i&1]+sorted(a[i]+1 for i in range(n) if mask>>i&1)
            if best is None or res<best: best=res
        out+=best
    return out
def g06(r):
    while True:
        n=r.randint(1,3); m=r.randint(1,3); q=r.randint(1,4)
        a=[r.randint(1,5) for _ in range(n)]; d=[r.randint(1,20) for _ in range(n)]
        o=sorted(range(n),key=lambda i:d[i]); c=0; ok=True
        for i in o:
            c+=a[i]; ok&=c<=d[i]
        if ok: break
    l=[r.randint(1,5) for _ in range(m)]; t=[r.randint(1,30) for _ in range(q)]
    return f"1\n{n} {m} {q}\n{' '.join(map(str,a))}\n{' '.join(map(str,d))}\n{' '.join(map(str,l))}\n{' '.join(map(str,t))}\n"
def b06(inp):
    v=list(map(int,inp.split())); n,m,q=v[1:4]; p=4
    a=v[p:p+n]; d=v[p+n:p+2*n]; l=v[p+2*n:p+2*n+m]; t=v[p+2*n+m:]
    best=[None]*(m+1)  # earliest finish of episode j over feasible full schedules
    for perm in itertools.permutations(range(n)):
        for pos in itertools.combinations(range(n+m),n):
            cur=0; ti=0; ei=0; ok=True; fin=[]
            for s in range(n+m):
                if s in pos:
                    i=perm[ti]; ti+=1; cur+=a[i]; ok&=cur<=d[i]
                else:
                    cur+=l[ei]; ei+=1; fin.append(cur)
            if ok:
                for j in range(m):
                    if best[j] is None or fin[j]<best[j]: best[j]=fin[j]
    return [sum(1 for j in range(m) if best[j]<=tt) for tt in t]
def g07(r):
    t=r.randint(1,3); s=f"{t}\n"
    for _ in range(t):
        n=r.randint(1,6); q=r.randint(1,5)
        s+=f"{n} {q}\n"+" ".join(str(r.randint(1,30)) for _ in range(n))+"\n"
        for _ in range(q):
            x=r.randint(1,n); y=r.randint(x,n); s+=f"{x} {y}\n"
    return s
def b07(inp):
    v=list(map(int,inp.split())); t=v[0]; p=1; out=[]
    for _ in range(t):
        n,q=v[p],v[p+1]; a=v[p+2:p+2+n]; p+=2+n
        for _ in range(q):
            x,y=v[p],v[p+1]; p+=2; seg=a[x-1:y]
            if len(set(seg))==1: out.append(0); continue
            out.append(max(m for m in range(1,31) if len(set(e%m for e in seg))==1))
    return out
def g08(r):
    t=r.randint(1,4); s=f"{t}\n"
    for _ in range(t):
        n=r.randint(0,5); m=r.randint(0,5)
        if n+m==0: n=1
        s+=f"{n} {m} {r.randint(1,n+m)}\n"
    return s
def bal(st):
    P=[0]
    for ch in st: P.append(P[-1]+(1 if ch=='0' else -1))
    return max(P)-min(P)
def c08(inp,outp):
    v=list(map(int,inp.split())); toks=outp.split()
    if len(toks)!=v[0]: return False
    for i in range(v[0]):
        n,m,k=v[1+3*i:4+3*i]
        poss=any(bal(''.join(c))==k for c in set(itertools.permutations('0'*n+'1'*m)))
        o=toks[i]
        if o=='-1':
            if poss: return False
        else:
            if not poss or len(o)!=n+m or o.count('0')!=n or o.count('1')!=m or bal(o)!=k: return False
    return True
def g10(r):
    t=r.randint(1,3); s=f"{t}\n"
    for _ in range(t):
        n=r.randint(2,8); m=r.randint(2,5); s+=f"{n} {m}\n"+" ".join(str(r.randint(1,n)) for _ in range(m))+"\n"
    return s
def b10(inp):
    v=list(map(int,inp.split())); t=v[0]; p=1; out=[]
    for _ in range(t):
        n,m=v[p],v[p+1]; a=v[p+2:p+2+m]; p+=2+m
        out.append(sum(1 for i in range(1,n) for c1 in range(m) for c2 in range(m) if c1!=c2 and a[c1]>=i and a[c2]>=n-i))
    return out
P={'01-2075D':(g01,b01),'02-2053C':(g02,b02),'04-2070C':(g04,b04),'05-2046B':(g05,b05),'06-2052J':(g06,b06),'07-2050F':(g07,b07),'08-2065E':(g08,None),'10-2075C':(g10,b10)}
def run(cmd,inp):
    r=subprocess.run(cmd,input=inp,capture_output=True,text=True,timeout=60); return r.returncode,r.stdout,r.stderr
def one(name,seed):
    g,b=P[name]; r=random.Random(seed*1000+hash(name)%997); inp=g(r)
    ri=run([C,'run',f'{D}/{name}.cp'],inp); rc=run([f'{B}/{name}'],inp)
    if name=='08-2065E':
        ok=ri[0]==0 and rc[0]==0 and c08(inp,ri[1]) and c08(inp,rc[1]); exp='checker'
    else:
        exp=' '.join(map(str,b(inp))); ok=ri[0]==0 and rc[0]==0 and ri[1].split()==exp.split() and rc[1].split()==exp.split()
    return ok,name,seed,inp,ri,rc,exp
names=sys.argv[1:] or list(P)
for name in names:
    with ThreadPoolExecutor(8) as ex: res=list(ex.map(lambda s:one(name,s),range(200)))
    bad=[x for x in res if not x[0]]
    print(name,'pass',200-len(bad),'fail',len(bad),flush=True)
    for x in bad[:2]:
        print(' seed',x[2],'IN:',x[3].replace('\n','|'),'INTERP:',x[4][0],x[4][1].replace('\n','|'),x[4][2][:300],'CPP:',x[5][0],x[5][1].replace('\n','|'),'EXP:',x[6],flush=True)
