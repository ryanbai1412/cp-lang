import random
r=random.Random(7); W={}
def w(pid,s): open(f'{pid}.in','w').write(s)
# 1931F
n,k=1000,200; order=list(range(1,n+1)); r.shuffle(order); L=[f"1\n{n} {k}"]
for _ in range(k):
    a=r.choice(order); L.append(' '.join(map(str,[a]+[x for x in order if x!=a])))
w('1931F','\n'.join(L)+'\n')
w('1931G',"1\n1000000 1000000 1000000 1000000\n")
n=300000; w('1929D',f"1\n{n}\n"+''.join(f"{i} {i+1}\n" if i%2 else f"{r.randint(1,i)} {i+1}\n" for i in range(1,n)))
n=200000; w('1923E',f"1\n{n}\n"+' '.join(str(r.randint(1,3)) for _ in range(n))+"\n"+''.join(f"{i} {i+1}\n" for i in range(1,n)))
n,m=10**6,200000; L=[f"1\n{n} {m}"]
for _ in range(m):
    l=r.randint(1,n); L.append(f"{l} {min(n,l+r.randint(0,20))}")
w('1932F','\n'.join(L)+'\n')
w('1928C',"100\n"+''.join(f"{735134400+1+(i%7)} {r.randint(1,5)}\n" for i in range(100)))
n,m=100000,200000; es=set()
while len(es)<m:
    u,v=r.randint(1,n),r.randint(1,n)
    if u!=v and (v,u) not in es: es.add((u,v))
w('1927F',f"1\n{n} {m}\n"+''.join(f"{u} {v} {r.randint(1,10**6)}\n" for u,v in es))
n=m=1000; L=["1",f"{n} {m}"]
for i in range(n): L.append(' '.join('0' if j==m-1 or (i,j)==(0,0) or r.random()>0.15 else '1' for j in range(m)))
w('1933F','\n'.join(L)+'\n')
w('1930D1',"1\n100\n"+''.join(r.choice('01') for _ in range(100))+"\n")
n=300000; w('1922D',f"1\n{n}\n"+' '.join(str(r.randint(1,10**9)) for _ in range(n))+"\n"+' '.join(str(r.randint(1,2*10**9//3)) for _ in range(n))+"\n")
