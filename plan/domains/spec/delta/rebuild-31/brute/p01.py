import sys,random
from collections import deque
def gen(r):
    T=r.randint(1,3);out=[str(T)]
    for _ in range(T):
        n=r.randint(1,6);m=r.randint(1,n);A='abc'[:r.randint(1,3)]
        out+=[f"{n} {m}",''.join(r.choice(A) for _ in range(n)),''.join(r.choice(A) for _ in range(m))]
    return '\n'.join(out)+'\n'
def solve(inp):
    it=iter(inp.split());T=int(next(it));res=[]
    for _ in range(T):
        n=int(next(it));m=int(next(it));s=next(it);t=next(it)
        seen={s};dq=deque([s])
        while dq:
            x=dq.popleft()
            nb=[x[:i]+x[i+1:] for i in range(len(x))]
            nb+=[x[:l]+''.join(sorted(x[l:r+1]))+x[r+1:] for l in range(len(x)) for r in range(l+1,len(x))]
            for y in nb:
                if len(y)>=m and y not in seen: seen.add(y);dq.append(y)
        res.append('YES' if t in seen else 'NO')
    return '\n'.join(res)
