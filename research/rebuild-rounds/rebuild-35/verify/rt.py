# Random tests: 200 per file against a Python brute, on `cpl run`, the -O2 build and the
# sanitizer build (RT_BIN holds NAME and NAME.asan from `cpl build` + g++).
import sys, random, subprocess, os, itertools, math
from fractions import Fraction
from concurrent.futures import ThreadPoolExecutor
HERE=os.path.dirname(os.path.abspath(__file__)); D=os.path.dirname(HERE); W=os.environ.get('RT_WORK','/tmp/rt35'); B=os.environ.get('RT_BIN',W+'/bin'); CPL=os.environ.get('CPL','cpl')
def clog(k):
    b=0
    while (1<<b)<k: b+=1
    return b
def g01(r):
    n=r.randint(1,10); I=r.randint(1,4); a=[r.randint(0,r.choice([3,10,10**9])) for _ in range(n)]
    return f"{n} {I}\n{' '.join(map(str,map(int,a)))}\n"
def b01(s, rev=False):
    t=list(map(int,s.split())); n,I=t[0],t[1]; a=t[2:]; vs=sorted(set(a)); best=n
    for i in range(len(vs)):
        for j in range(i,len(vs)):
            K=j-i+1; need=n*clog(K)+(32*K if rev else 0)
            if need<=8*I: best=min(best,sum(1 for x in a if x<vs[i] or x>vs[j]))
    return f"{best}\n"
def g01r(r):
    s=g01(r); t=s.split(); t[1]=str(r.randint(4,40)); n=int(t[0]); return f"{n} {t[1]}\n{' '.join(t[2:])}\n"
def g02(r):
    n=r.randint(1,8); p=list(range(1,n+1)); r.shuffle(p); p=[x if r.random()<0.5 else 0 for x in p]
    return f"{n}\n{' '.join(map(str,p))}\n"
def b02(s, rev=False):
    t=list(map(int,s.split())); n=t[0]; p=t[1:]; miss=[x for x in range(1,n+1) if x not in p]; best=None
    for perm in itertools.permutations(miss):
        it=iter(perm); q=[x if x else next(it) for x in p]
        c=sum(1 for i in range(n-1) if q[i]%2!=q[i+1]%2)
        if rev and n>1 and q[0]%2!=q[-1]%2: c+=1
        best=c if best is None else min(best,c)
    return f"{best}\n"
def g04(r):
    T=r.randint(1,3); out=[str(T)]
    for _ in range(T):
        n=r.randint(2,8); es=set()
        perm=list(range(1,n+1)); r.shuffle(perm)
        for i in range(1,n): es.add(tuple(sorted((perm[i],perm[r.randrange(i)]))))
        for _ in range(r.randint(0,n)):
            u,v=r.sample(range(1,n+1),2); es.add(tuple(sorted((u,v))))
        es=list(es); r.shuffle(es); out.append(f"{n} {len(es)}"); out+= [f"{u} {v}" if r.random()<.5 else f"{v} {u}" for u,v in es]
    return "\n".join(out)+"\n"
def g05(r):
    if r.random()<0.75: return f"{r.randint(1,60)} {r.choice([r.randint(2,100), r.randint(2,10**12), 2**r.randint(1,39), r.choice([2,3,5,7,999983])**r.randint(1,2)])}\n"
    return f"{r.choice([10**18, r.randint(1,10**18), 10**18-1])} {r.choice([r.randint(2,10**12), 999999999989, 2, 10**12, 999983*999979])}\n"
def b05(s):
    n,b=map(int,s.split())
    if n<=60:
        f=math.factorial(n); c=0
        while f%b==0: f//=b; c+=1
        return f"{c}\n"
    fs={}; x=b; p=2
    while p*p<=x:
        while x%p==0: fs[p]=fs.get(p,0)+1; x//=p
        p+=1
    if x>1: fs[x]=fs.get(x,0)+1
    def leg(p):
        c=0; q=p
        while q<=n: c+=n//q; q*=p
        return c
    return f"{min(leg(p)//e for p,e in fs.items())}\n"
def g06(r):
    n=r.randint(1,6); al=r.choice(["ab","abc","aA0"]); w=["".join(r.choice(al) for _ in range(r.randint(1,6))) for _ in range(n)]
    return f"{n}\n{' '.join(w)}\n"
def b06(s):
    t=s.split(); res=t[1]
    for w in t[2:]:
        k=max(k for k in range(0,min(len(w),len(res))+1) if res.endswith(w[:k]) )
        res+=w[k:]
    return res+"\n"
def g07(r):
    n=r.randint(1,20); s="".join(r.choice("()()LRa") for _ in range(n)); return f"{n}\n{s}\n"
def b07(s):
    n,cmd=s.split(); txt={}; cur=0; out=[]
    for c in cmd:
        if c=='L': cur=max(0,cur-1)
        elif c=='R': cur+=1
        else: txt[cur]=c
        bal=0; mx=0; ok=True
        for i in sorted(txt):
            if txt[i]=='(': bal+=1
            elif txt[i]==')': bal-=1
            if bal<0: ok=False
            mx=max(mx,bal)
        out.append(str(mx if ok and bal==0 else -1))
    return " ".join(out)+"\n"
def g08(r):
    n=r.randint(1,6); m=r.randint(1,6); g=[[('*' if r.random()<0.3 else '.') for _ in range(m)] for _ in range(n)]
    r0=r.randint(1,n); c0=r.randint(1,m); g[r0-1][c0-1]='.'
    x=r.choice([0,1,2,3,10**9]); y=r.choice([0,1,2,3,10**9])
    return f"{n} {m}\n{r0} {c0}\n{x} {y}\n"+"\n".join("".join(row) for row in g)+"\n"
def b08(s):
    t=s.split(); n,m,r0,c0,x,y=map(int,t[:6]); g=t[6:]; X=min(x,40); Y=min(y,40)
    from collections import deque
    st=(r0-1,c0-1,0,0); seen={st}; q=deque([st])
    while q:
        a,b,l,rr=q.popleft()
        for da,db,dl,dr in ((1,0,0,0),(-1,0,0,0),(0,-1,1,0),(0,1,0,1)):
            na,nb,nl,nr=a+da,b+db,l+dl,rr+dr
            if 0<=na<n and 0<=nb<m and g[na][nb]=='.' and nl<=X and nr<=Y and (na,nb,nl,nr) not in seen:
                seen.add((na,nb,nl,nr)); q.append((na,nb,nl,nr))
    return f"{len({(a,b) for a,b,_,_ in seen})}\n"
def g09(r):
    n=r.randint(1,9); return f"{n}\n{' '.join(str(r.choice([r.randint(1,10),r.randint(1,10**6)])) for _ in range(n))}\n"
def b09(s):
    t=list(map(int,s.split())); n=t[0]; a=t[1:]; i=0; res=[]
    while i<n:
        best=None; bj=i
        for j in range(i,n):
            av=Fraction(sum(a[i:j+1]),j-i+1)
            if best is None or av<=best: best=av; bj=j
        res+= [best]*(bj-i+1); i=bj+1
    return "".join(f"{float(v):.9f}\n" for v in res)
def g10(r):
    n=r.randint(1,7); C=r.choice([5,10**6]); K=r.choice([5,10**9])
    pts="\n".join(f"{r.randint(1,C)} {r.randint(1,C)}" for _ in range(n))
    return f"{n}\n{pts}\n{' '.join(str(r.randint(1,K)) for _ in range(n))}\n{' '.join(str(r.randint(1,K)) for _ in range(n))}\n"
def b10(s):
    t=list(map(int,s.split())); n=t[0]; xy=[(t[1+2*i],t[2+2*i]) for i in range(n)]; c=t[1+2*n:1+3*n]; k=t[1+3*n:]
    INF=float('inf'); d=[INF]*(n+1); d[0]=0; used=[False]*(n+1); tot=0
    def w(i,j):
        if i==0: return c[j-1]
        if j==0: return c[i-1]
        return (k[i-1]+k[j-1])*(abs(xy[i-1][0]-xy[j-1][0])+abs(xy[i-1][1]-xy[j-1][1]))
    for _ in range(n+1):
        u=min((v for v in range(n+1) if not used[v]), key=lambda v:d[v]); used[u]=True; tot+=d[u]
        for v in range(n+1):
            if not used[v]: d[v]=min(d[v],w(u,v))
    return f"{tot}\n"
P={'01-1198A':(g01,b01,None),'01-1198A-rev':(g01r,lambda s:b01(s,True),None),'02-1286A':(g02,b02,None),'02-1286A-rev':(g02,lambda s:b02(s,True),None),
 '04-1176E':(g04,lambda s:"",'chk04'),'05-1114C':(g05,b05,None),'06-1200E':(g06,b06,None),'07-1263E':(g07,b07,None),'08-1063B':(g08,b08,None),'09-1299C':(g09,b09,'chk09'),'10-1245D':(g10,b10,'chk10')}
def judge(name,chk,inp,out,ans,dirp,i,mode):
    if chk is None: return out.split()==ans.split()
    op=f"{dirp}/{i}.{mode}.got"; open(op,'w').write(out)
    return subprocess.run(['python3',f'{HERE}/{chk}.py',f"{dirp}/{i}.in",op,f"{dirp}/{i}.ans"],capture_output=True).returncode==0
def one(name,i):
    g,b,chk=P[name]; dirp=f'{W}/rt/{name}'; r=random.Random(hash((name,i))&0xffffffff)
    inp=g(r); ans=b(inp); open(f"{dirp}/{i}.in",'w').write(inp); open(f"{dirp}/{i}.ans",'w').write(ans)
    res={}
    for mode,cmd in (('interp',[CPL,'run',f"{D}/{name}.cp",f"{dirp}/{i}.in"]),('native',[f"{B}/{name}"]),('asan',[f"{B}/{name}.asan"])):
        p=subprocess.run(cmd,input=inp if mode!='interp' else None,capture_output=True,text=True,timeout=120)
        ok=p.returncode==0 and judge(name,chk,inp,p.stdout,ans,dirp,i,mode)
        if mode=='asan' and p.stderr.strip(): ok=False
        res[mode]=(ok,p.stdout,p.stderr[-300:])
    agree=res['interp'][1]==res['native'][1]
    return i,res,agree
if __name__=='__main__':
    names=sys.argv[1:] or list(P)
    for name in names:
        os.makedirs(f'{W}/rt/{name}',exist_ok=True)
        with ThreadPoolExecutor(8) as ex: rs=list(ex.map(lambda i: one(name,i), range(200)))
        fails=[(i,{m:(v[0],v[2]) for m,v in res.items() if not v[0]}) for i,res,ag in rs if not all(v[0] for v in res.values())]
        dis=[i for i,res,ag in rs if not ag]
        print(name, 'pass', sum(1 for i,res,ag in rs if all(v[0] for v in res.values())), '/200 disagree', len(dis), 'first fails', fails[:2], flush=True)
