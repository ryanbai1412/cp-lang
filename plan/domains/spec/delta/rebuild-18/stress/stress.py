# usage: python3 stress.py <cfid> <file.cp> <compiled C++ binary> [count]; CPL=path to cpl
import sys, random, subprocess, brutes
import os; C=os.environ.get('CPL', 'cpl')
name=sys.argv[1]; src=sys.argv[2]; binp=sys.argv[3]; N=int(sys.argv[4]) if len(sys.argv)>4 else 200
cf=name; gen=getattr(brutes,'g'+cf); br=getattr(brutes,'b'+cf,None); chk=getattr(brutes,'chk'+cf,None)
norm=lambda s:' '.join(s.split())
bad=0
for seed in range(N):
    r=random.Random(seed); inp=gen(r)+'\n'
    a=subprocess.run([C,'run',src],input=inp,capture_output=True,text=True,timeout=60)
    b=subprocess.run([binp],input=inp,capture_output=True,text=True,timeout=60)
    issues=[]
    if a.returncode: issues.append('interp rc=%d %s'%(a.returncode,a.stderr[:300]))
    if b.returncode: issues.append('cpp rc=%d'%b.returncode)
    if norm(a.stdout)!=norm(b.stdout): issues.append('interp!=cpp')
    if chk:
        if not chk(inp,a.stdout): issues.append('interp fails checker')
        if not chk(inp,b.stdout): issues.append('cpp fails checker')
    elif br:
        e=br(inp)
        if norm(a.stdout)!=norm(e): issues.append('interp!=brute')
        if norm(b.stdout)!=norm(e): issues.append('cpp!=brute')
    if issues:
        bad+=1
        if bad<=3: print('seed',seed,issues,'\nINPUT:\n'+inp+'interp:'+a.stdout[:200]+'\ncpp:'+b.stdout[:200], flush=True)
print(f'{name}: {N-bad}/{N} clean')
