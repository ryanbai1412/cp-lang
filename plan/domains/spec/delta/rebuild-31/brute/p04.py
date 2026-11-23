def gen(r):
    T=r.randint(1,2);out=[str(T)]
    for _ in range(T):
        n=r.randint(1,8);m=r.randint(1,8);out+=[f"{n} {m}",''.join(r.choice('abc') for _ in range(n))]
        for _ in range(m):
            l=r.randint(1,n);rr=r.randint(l,n)
            if r.random()<0.5: out.append(f"1 {l} {rr} {r.choice([r.randint(1,3),r.randint(1,10**9)])}")
            else: out.append(f"2 {l} {rr}")
    return '\n'.join(out)+'\n'
def solve(inp):
    it=iter(inp.split());T=int(next(it));res=[]
    for _ in range(T):
        n=int(next(it));m=int(next(it));s=[ord(c)-97 for c in next(it)]
        for _ in range(m):
            t=int(next(it));l=int(next(it))-1;r=int(next(it))
            if t==1:
                x=int(next(it))
                for i in range(l,r): s[i]=(s[i]+x)%26
            else:
                sub=s[l:r];ok=all(sub[i]!=sub[i+1] for i in range(len(sub)-1)) and all(sub[i]!=sub[i+2] for i in range(len(sub)-2))
                res.append('YES' if ok else 'NO')
    return '\n'.join(res)
